use super::{
    block_cache_sync_all, get_block_cache, BlockDevice, DirEntry, DiskInode, DiskInodeType,
    EasyFileSystem, DIRENT_SZ,
};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};
use alloc::vec;
/// Virtual filesystem layer over easy-fs
pub struct Inode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<EasyFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
}

impl Inode {
    /// Create a vfs inode
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<EasyFileSystem>>,
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_device,
        }
    }
    /// Call a function over a disk inode to read it
    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, f)
    }
    /// Call a function over a disk inode to modify it
    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .modify(self.block_offset, f)
    }
    /// Find inode under a disk inode by name
    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            //同时也要确保inode_id不是-1，因为-1表示被伪删除了
            if dirent.name() == name && dirent.inode_id() != u32::MAX {
                return Some(dirent.inode_id() as u32);
            }
        }
        None
    }
    /// 返回对应direntry的偏移量即 i值
    fn find_direntry(&self,name:&str,disk_inode:&DiskInode) -> usize{
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name{
                return i as usize;
            }
        }
        return usize::MAX as usize;
    } 

    /// 通过inode_id找到所有有关的direntry的偏移量
    fn find_direntry_by_inode_id(&self,inode_id:u32,disk_inode:&DiskInode) -> Vec<usize>{
        assert!(disk_inode.is_dir());
        let mut v:Vec<usize> = vec![];
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.inode_id() == inode_id {
                v.push(i as usize);
            }
        }
        return v;
    }

    fn find_inode_id_by_block(&self,block_id:usize,block_offset:usize,disk_inode:&DiskInode) -> Option<u32>{
        let fs = self.fs.lock();
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.inode_id() != u32::MAX {
                let (block_id_inode, block_offset_inode) = fs.get_disk_inode_pos(dirent.inode_id());
                if block_id_inode == (block_id as u32) && block_offset_inode == block_offset{
                    return Some(dirent.inode_id());
                }
            }
        }
        None
    }
    //将对应的位置的dir更新
    fn update_dir_entry(&self,name:&str,inode_id:u32,index:usize,disk_inode:&mut DiskInode){
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        assert!(index<file_count);
        let mut dirent = DirEntry::new(name,inode_id);
        assert_eq!(
            disk_inode.write_at(DIRENT_SZ * index, dirent.as_bytes_mut(), &self.block_device,),
            DIRENT_SZ,
        );
    }

    /// Find inode under current inode by name
    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(name, disk_inode).map(|inode_id| {
                let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                Arc::new(Self::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_device.clone(),
                ))
            })
        })
    }
    /// Increase the size of a disk inode
    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<EasyFileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }
    /// Create inode under current inode by name
    pub fn create(&self, name: &str) -> Option<Arc<Inode>> {
        let mut fs = self.fs.lock();
        let op = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, root_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();
        // initialize inode
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
                new_inode.initialize(DiskInodeType::File);
            });
        self.modify_disk_inode(|root_inode| {
            //这里的逻辑需要变动，首先确定一下是否有被伪删除的节点
            //如果存在，则将对应内容写到伪删除的节点上
            let vec_dir = self.find_direntry_by_inode_id(u32::MAX,root_inode);
            if vec_dir.len() != 0 {
                self.update_dir_entry(name,new_inode_id,vec_dir[0],root_inode);
            }else{
                // append file in the dirent
                let file_count = (root_inode.size as usize) / DIRENT_SZ;
                let new_size = (file_count + 1) * DIRENT_SZ;
                // increase size
                self.increase_size(new_size as u32, root_inode, &mut fs);
                // write dirent
                let dirent = DirEntry::new(name, new_inode_id);
                root_inode.write_at(
                    file_count * DIRENT_SZ,
                    dirent.as_bytes(),
                    &self.block_device,
                );
            }
        });

        let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
        block_cache_sync_all();
        // return inode
        Some(Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        )))
        // release efs lock automatically by compiler
    }
    ///获得对应的block_id
    pub fn get_block_id(&self)->usize{
        return self.block_id;
    }
    ///获取对应的block_offset
    pub fn get_block_offset(&self)->usize{
        return self.block_offset;
    }
    /// 实现文件的硬连接,node_before是需要连接的文件，newpath则是需要连接到的路径
    pub fn linkat(&self,old_path:&str,new_path:&str) -> isize{
        let mut _fs =self.fs.lock();
        let mut old_inode_id:u32= u32::MAX;
        
        //先找到oldpath对应的inode_id
        if let Some(inode_id) = self.read_disk_inode(|disk_inode| {
            assert!(disk_inode.is_dir());
            self.find_inode_id(old_path, disk_inode)
        }){
            old_inode_id = inode_id;
        }else{
            return -1;
        }
        
        //检查是否合法,是否返回了一个正常的数字
        if old_inode_id == u32::MAX {
            return -1;
        }

        //去寻找一下是否存在newpath对应的文件，不存在则开始进行创建，如果存在的话则返回-1
        let op = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id(new_path, root_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return -1;
        }

        //开始将我们newpath连接到对应inode上面去，这样就可以实现硬连接了
        self.modify_disk_inode(|root_inode| {

            let vec_dir = self.find_direntry_by_inode_id(u32::MAX,root_inode);
            if vec_dir.len() != 0 {
                self.update_dir_entry(new_path,old_inode_id,vec_dir[0],root_inode);
            }else{
                // append file in the dirent
                let file_count = (root_inode.size as usize) / DIRENT_SZ;
                let new_size = (file_count + 1) * DIRENT_SZ;
                // increase size
                self.increase_size(new_size as u32, root_inode, &mut _fs);
                // write dirent
                let dirent = DirEntry::new(new_path, old_inode_id);
                root_inode.write_at(
                    file_count * DIRENT_SZ,
                    dirent.as_bytes(),
                    &self.block_device,
                );
            }
        });

        return 0;
        

    }
   
    /// 进行解链接，值得一提的是采用的是伪删除，即目录项不会真的删掉，而是会使其inode_num指向-1，即表示不存在，这样做的好处是不需要动root_node的size，
    /// 坏处是可能被攻击，但是我们可以通过修改生成的逻辑，使生成文件的时候先寻找有无被伪删除的目录项，将新目录项的内容写到对应的位置即可
    pub fn unlinkat(&self,name:&str) -> isize{
        //分两种情况 一种是只剩最后一个目录项了，那么此时就要删除inode并清空文件内容，如果不是最后一个目录项，那么只需要删除该目录项即可
        //首先得到对应路径下目录项的文件inode_id
        let mut _fs = self.fs.lock();
        let mut inode_id_file:u32 = u32::MAX;
        if let Some(inode_id) = self.read_disk_inode(|disk_inode| {
            assert!(disk_inode.is_dir());
            self.find_inode_id(name, disk_inode)
        }){
            inode_id_file = inode_id;
        }else{
            return -1;
        }

        let getalldir = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_direntry_by_inode_id(inode_id_file, root_inode)
        };  

        let getdir = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_direntry(name, root_inode)
        };

        let index = self.read_disk_inode(getdir);

        let delete = |root_inode: &mut DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.update_dir_entry(name,u32::MAX,index,root_inode)
        };

        let vec_dir = self.read_disk_inode(getalldir);

        if vec_dir.len()>=2 {
            self.modify_disk_inode(delete);
        }else if vec_dir.len() == 1{
            //此时既要删除目录项，又需要删除文件
            assert!(inode_id_file != u32::MAX);
            let (block_id, block_offset) = _fs.get_disk_inode_pos(inode_id_file);
            let inode:Inode = Inode::new(
                block_id,
                block_offset,
                self.fs.clone(),
                self.block_device.clone(),
            );
            //这里不能直接调用，一定要先把锁去掉，切记
            drop(_fs);
            inode.clear();
            self.modify_disk_inode(delete);
        }

        if vec_dir.len() < 1 {
            return -1;
        }
        0
    }
    ///三元组分别代表ino，nlink和文件类型，0表示null，1表示目录，2表示文件
    pub fn stat(&self,block_id:usize,block_offset:usize) -> (u32,u32,u32){
        
        //下段代码只用于测试
        // let find_1 = |root_inode: &DiskInode| {
        //     // assert it is a directory
        //     assert!(root_inode.is_dir());
        //     // has the file been created?
        //     self.find_inode_id(&"fname2",root_inode)
        // }; 
        // let find_2 = |root_inode: &DiskInode| {
        //     // assert it is a directory
        //     assert!(root_inode.is_dir());
        //     // has the file been created?
        //     self.find_inode_id(&"linkname0",root_inode)
        // }; 

        // let node1 = self.read_disk_inode(find_1);
        // let node2 = self.read_disk_inode(find_2);
        // assert!(node1.unwrap()==node2.unwrap());

        let fs = self.fs.lock();
        let mut inode_id:u32 = u32::MAX;

        drop(fs);//又是这个抽象问题，一定要记住mutex在不同函数间也必须确保唯一性
        let find_inode_id = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id_by_block(block_id,block_offset,root_inode)
        };
        //获取到对应的
        match self.read_disk_inode(find_inode_id){
            Some(inode_id_disk) => {
                inode_id = inode_id_disk;
            },
            None => {
                return (u32::MAX,u32::MAX,u32::MAX);
            }
        }
        //必须要找到对应的inode_id，否则就返None，因为没有这样的文件存在
        if inode_id == u32::MAX {
            return (u32::MAX,u32::MAX,u32::MAX);
        }

        let find_nlink = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_direntry_by_inode_id(inode_id,root_inode)
        };
    
        //获取到连接数量
        let vec_dir: Vec<usize> = self.read_disk_inode(find_nlink);
        let nlink = vec_dir.len();

        //因为我们只在同一目录下，所以只可能是文件类型不可能是目录
        return (inode_id,nlink as u32,2);
    }
    /// List inodes under current inode
    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            let mut v: Vec<String> = Vec::new();
            for i in 0..file_count {
                let mut dirent = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                //只列出没有被伪删除的目录项
                if dirent.inode_id() != u32::MAX {
                    v.push(String::from(dirent.name()));
                }
            }
            v
        })
    }
    /// Read data from current inode
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, buf, &self.block_device))
    }
    /// Write data to current inode
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let size = self.modify_disk_inode(|disk_inode| {
            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }
    /// Clear the data in current inode
    pub fn clear(&self) {
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|disk_inode| {
            let size = disk_inode.size;
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
            for data_block in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(data_block);
            }
        });
        block_cache_sync_all();
    }
}
