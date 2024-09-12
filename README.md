结合embedding+chatgpt 阅读代码

先将项目所有文件计算embedding，并储存在项目目录的`.readit`中。

提问时先查询embeding，再用查询的结果询问chatgpt，节省大量token。

![init project](./img/init.png)

![ask something](./img/ask.png)


```
cargo build -r
```

```
readit -h
```

# TODO 
- [ ] embedding数据库查找不到数据时候的处理
- [ ] 记录的文件路径改为项目相对路径，解决一旦移动项目就无法使用的问题
