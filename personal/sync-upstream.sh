#!/bin/sh
# main 保持与上游 qingjian-team/qingjian 一致，个人改动在 personal 分支上 rebase 到最新上游，
# 再打包成 pkg 装到 /Library/Input Methods（覆盖官方版的位置，postinstall 自动注册，不用注销）
set -e
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
git fetch upstream
git push origin upstream/main:main
git rebase upstream/main personal
git push --force-with-lease origin personal
apps/macos/scripts/bundle.sh --pkg
pkg="$(ls -t target/pkg/qingjian-*-macos-*.pkg | head -1)"
sudo installer -pkg "$pkg" -target /
# postinstall 注册过，但装完后系统偶尔仍把青简从输入源里摘掉，以当前用户身份再注册一次
"/Library/Input Methods/Qingjian.app/Contents/MacOS/qingjian-macos" --register
