#!/bin/bash

# 获取当前目录下的.cpio文件数量
count=$(ls -1 *.cpio 2>/dev/null | wc -l)

# 检查.cpio文件的数量是否为一个
if [ $count -eq 1 ]; then
	:
elif [ "$count" -gt 1 ]; then
	echo -e "当前目录下 .cpio 文件怎么TM有好几个呢\U1F618"
	exit 1 # 使用非零退出码立即退出脚本
fi

# 获取当前目录中所有.cpio文件
files=$(ls *.cpio 2>/dev/null)

# 检查是否存在.cpio文件
if [ -z "$files" ]; then
	echo -e "然而当前文件夹中并没有 .cpio 文件，莫非是在耍洒家\U1F643"
	exit 1
fi

# 遍历所有.cpio文件并重命名
for file in $files; do
	if [ "$file" != "core.cpio" ]; then
		mv "$file" "core.cpio"
		echo -e "初始化已完成，用一次就行\U1F607"
		break # 如果你只想重命名一个文件，使用此行
	else
		echo -e "文件名 core.cpio 已经存在力，你还用个锤子\U1F60B"
	fi
done
