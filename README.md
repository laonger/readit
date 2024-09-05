结合embedding+chatgpt 阅读代码

先将项目所有文件计算embedding，并储存在项目目录的`.readit`中。

提问时先查询embeding，再用查询的结果询问chatgpt，节省大量token。

```
cargo build -r
```

```
readit -h
```
