# Portunex 重建版 · 前端控制台

免构建的单文件 SPA(纯 HTML + 原生 JS,dark 主题),对接重建的 Rust 后端。
**不需要 npm / 构建工具**,静态托管即可。

## 功能
- 登录 / 注册
- 概览:积分余额、账号、我的订阅、兑换码充值
- API Keys:创建 / 列表 / 停用
- 订阅:列出套餐、积分购买
- 管理后台(role=admin):建兑换码 / 套餐 / provider / 模型别名,查看 providers / 别名 / 用户
- OIDC:签发并查看 RS256 id_token,discovery / jwks 直达

## 运行
```bash
# 1) 先起后端(见仓库根 README),假设监听 :8091
# 2) 静态托管本目录
cd frontend
python3 -m http.server 8092
# 浏览器打开 http://localhost:8092  ——  页面右上角可改后端地址
```
> 后端已配 CORS `*`,前端从任意端口/源都能调。首个 admin 账号见后端首启日志
> (admin@portunex.local + 随机密码),或自行注册。

## 说明
- 这是**照重建后端的真实接口**手写的前端(可编辑源码),不是原产品逆向出来的压缩包。
- 与后端已验证的端点一一对应:`/auth/*`、`/me`、`/api-keys*`、`/redeem`、
  `/subscriptions`、`/admin/*`、`/v1/models`、`/oidc/*`。
- 原产品前端是 React-Router v7/Remix + three.js 的大型 SPA;如需还原那套观感,
  按 `RECONSTRUCTION_BLUEPRINT.md` 第 5 节的页面清单另起 React 项目即可。
