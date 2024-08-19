## lab3实验报告

#### 编程作业实验结果

![img](file:///C:\Users\Hp\AppData\Local\Temp\QQ_1723860928800.png)

测试用例已经通过

实验思路：sys_spawn()采取的思路是仿照fork构建出来一个task，但是其中地址空间不必完全按照父进程构建，之后调用exec执行该进程的elf文件。stride（）的思路是，在inner里面放入priority和stride两个字段，并且写一个pass（）函数用于计算对应的pass值。在manager中写一个遍历所有任务来得到stride值最小任务的方法，并应用在processor切换任务的过程中。向前兼容task_info,getTime,mmap和unmmap基本和之前的实验差不多，不过需要注意的是，字段要放在inner中，因为TCB使用Arc指针的，不能生成可变引用。



#### 问答题：

1.（1）并不轮到p1，因为p2溢出了，这下p2反而更小（在8bit无符号整数的格式下）

（2）因为如果优先级最小是2的话，那么每次stride最小值最多只能添加big_stride/2，利用数学归纳法，在第一次增加stride值时必然成立，因为增加值不会超过bigStride/2，而其他stride值都是0，设第k次增加stride值时成立，那么在有k+1时，由于此时选定的是最小的stride0，所以即使加上bigStride/2，其他的在之前超过他的stride值也不会小于增加之后的stride0值。

（3）

```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        //如果self小于other但不满足定理则说明发生了溢出，如果self大于other且满足定理则说明self确实大于other
        if (self.0 > other.0 && (self.0 - other.0)<(255/2)) ||(self.0 < other.0 && (other.0 - self.0)>(255/2)){
            return Some(Ordering::Greater);
        }else if (self.0 < other.0 && (other.0 - self.0)<(255/2)) || (self.0 > other.0 && (self.0 - other.0)>(255/2)){     //与上面正好反过来
            return Some(Ordering::Less);
        }else{
            return None;
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```
