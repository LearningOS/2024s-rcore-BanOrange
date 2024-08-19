## lab4实验报告

#### 实验结果：

......

Usertests: Running ch6_file3
Usertests: Test ch6_file3 in Process 3 exited with code 0
ch6 Usertests passed!
Shell: Process 2 exited with code 0

代码思路：

（1）linkat，先找oldpath对应的inode_id，然后将其连接到new_path对应的目录项上

（2）unlinkat，先查找该inode_id一共有多少个连接，如果只有一个，那么就连同文件一起删掉，大于一个则将目录项的inode_id修改为u32::MAX表示被删除

（3）stat，从OSInode中得到block_id和block_offset并传入inode进行对比，如果找到了这开始依次查找Stat所需数据并返回



#### 问答题

1.root_inode起到一个根目录的作用，里面存储的都是目录项，可以帮助我们去索引文件，如果root_inode损坏，那么我们就缺少了name和inode_id的配对，即目录项，进而难以在文件系统中找到文件对应的inode，无法实现对文件的增删改查

2.(1)一个例子：利用cat和wc打开文件并进行字数统计，两者会建立管道进行数据传输，cat获取文件的内容并传递给wc进行统计

（2）可以将管道设计成为多入口多出口的形式，即每次生成一个进程都将其加入到这个多管道系统当中，只需要在多管道中指定入口和出口即可。或者采取命名管道的方式，将数据写入一个固定管道中，其他进程可以从这个管道中读取。
