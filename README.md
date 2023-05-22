# AxumとSvelteKitで作るツイッターライクなWebアプリ

# 採用技術
- Rust
- Axum (Webフレームワーク)
- SeaOrm (非同期処理が可能なORMライブラリ)
- SvelteKit　(フロントエンド)
- TypeScript 
- TailwindCSS
- Postgres
- Docker


# 前提条件（カッコ内のコマンドで確認できます＿
- Rustがインストールされている。(```cargo --version```)
- dockerが利用できる。(```docker --version```)
- npmが利用できる。(```node -v```か```npm -v```)
## 実行方法(MacOS)
1. リポジトリをgit cloneする
2. docker-composeを実行するとコンテナ上でPostgresとAxumサーバが立ち上がります。
3. ```cd front```でフロントエンドのディレクトリに移動。
4. ```npm install```等でnode_modulesをインストール
5. ```npm run dev```でlocalhost:8889で開発サーバが立ち上がります。