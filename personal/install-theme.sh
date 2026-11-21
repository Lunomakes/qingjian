#!/bin/sh
# 把仓库里的个人主题链接进青简数据文件夹，之后改仓库文件即生效（重开设置刷新主题列表）
set -e
src="$(cd "$(dirname "$0")/themes" && pwd)"
dst="$HOME/Library/Application Support/Qingjian/custom/themes"
mkdir -p "$dst"
for t in "$src"/*/; do ln -sfn "${t%/}" "$dst/$(basename "$t")"; done
ls -l "$dst"
