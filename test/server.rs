// 两个实例，某个具体协议的 inbound 就一定对应此具体协议
// 所以 local-proxy inbound 只需要socks就行，
// 其他协议的 inbound，通过 local-proxy#outbound => remote-proxy-server#inbound 测试