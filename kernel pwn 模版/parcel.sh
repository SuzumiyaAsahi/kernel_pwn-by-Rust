#!/bin/bash
cd core
find . -print0 |
	cpio --null -ov --format=newc |
	gzip -9 >core.cpio
mv ./core.cpio ../
