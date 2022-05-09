rust tun proxy implementation

大体设计

```plain
                                                                 |
------> socks inbound handler|                                   | ----> socks outbound handler
                             |   dispatcher(routing decision)    |
------> vmess inbound handler| =================================>| ----> vmess outbound handler
                             |                                   |
------> http inbound handler |                                   | ----> http outbound handler
                             |                                   |
------> tun inbound handler  |                                   |
                                                                 | ----> other protocols ...
```

按照约定的 `config.jsonc` 进行开发

有些 inbound protocol 会含有 tcp inbound 和 udp inbound

整体流程如下

1. inbound handler 分别listen addr
2. 收到请求后交给对应 Inbound handler 处理
3. 将 inbound handler 返回的 socket_A 交给dispatcher
4. dispatcher 根据路由选择 outbound handler
5. 调用 outbound handler，handler中负责连接server，并返回新的 socket_B
6. dispatcher的末尾，进行socket_A，socket_B 之间的数据转发。pipe(socket_A, socket_B)
