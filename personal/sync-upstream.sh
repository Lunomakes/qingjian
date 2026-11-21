#!/bin/sh
# main 保持与上游 qingjian-team/qingjian 一致，个人改动在 personal 分支上 rebase 到最新上游
set -e
cd "$(dirname "$0")/.."
git fetch upstream
git push origin upstream/main:main
git rebase upstream/main personal
git push --force-with-lease origin personal
