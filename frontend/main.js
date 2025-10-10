const { app, BrowserWindow } = require('electron');
const path = require('path');
const rust = require("./native/index");

(async () => {
  const result = await rust.delayedSum(10, 20);
  console.log("Result from Rust:", result);
})();

function createWindow() {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js')
    }
  });

  win.loadFile('renderer/dist/index.html');
}

app.whenReady().then(createWindow);
