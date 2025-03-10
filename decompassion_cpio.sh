#!/bin/bash
mkdir core
cd core
cpio -idmv <../core.cpio

# 有的不讲武德，写的是 .cpio 文件，但会发现解包后是一堆乱码，其实这根本是个 .cpio.gz 文件。
# 需要先 mv core.cpio ./core/core.cpio.gz
# 然后 gunzip core.cpio.gz
# 之后才可以 cpio -idmv <../core.cpio
