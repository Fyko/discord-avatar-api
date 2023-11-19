# discord-avatar-api

> redirects to a Discord user's avatar like github.com/Fyko.png

## Usage

https://davatar.fyko.net/$id.$format?size=$size

| Parameter | Description                                                                                                                                     | Default                   |
| :-------- | :---------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------ |
| `id`      | Discord user Id ([Turn on Developer Mode](https://support.discord.com/hc/en-us/articles/206346498-Where-can-I-find-my-User-Server-Message-ID-)) | required                  |
| `format`  | Image format (`png`, `webp`, `jpg`, `jpeg`, `gif`)                                                                                              | dynamic (`webp` or `gif`) |
| `size`    | Image size (`16`, `32`, `64`, `128`, `256`, `512`, `1024`, `2048`, `4096`)                                                                      | `512`                     |

## Recipes

| URL                                                                                       | Returns                             |
| :---------------------------------------------------------------------------------------- | :---------------------------------- |
| [/552734173803184128](https://davatar.fyko.net/552734173803184128)                        | ✅ /552734173803184128.webp?sze=512 |
| [/552734173803184128.png](https://davatar.fyko.net/552734173803184128.png)                | ✅ /552734173803184128.png?sze=512  |
| [/275357149477994498](https://davatar.fyko.net/552734173803184128) (assume avatar is gif) | ✅ /275357149477994498.gif?sze=512  |
