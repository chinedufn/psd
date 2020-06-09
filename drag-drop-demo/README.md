# Drag an Drop PSD Demo

The demo can be [viewed live](https://chinedufn.github.io/psd/drag-drop-demo/).

To run it locally

```
git clone git@github.com:chinedufn/psd.git
cd examples/drag-drop-browser

# ./build-dev.sh
./build-release.sh

npm install -g http-server # Or any other wasm compatible server
http-server -o -c -p 12000 public
```

![Demo screenshot](./demo-screenshot.png)
