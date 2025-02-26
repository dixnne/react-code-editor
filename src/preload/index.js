const { contextBridge, ipcRenderer } = require('electron');
import { electronAPI } from '@electron-toolkit/preload'

// Custom APIs for renderer
const api = {}

// Use `contextBridge` APIs to expose Electron APIs to
// renderer only if context isolation is enabled, otherwise
// just add to the DOM global.
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld('electron', {
      openFile: () => ipcRenderer.invoke("open-file"),
      openFolder: () => ipcRenderer.invoke("open-folder"),
      saveFile: (data) => ipcRenderer.invoke("save-file", data),
      saveFileAs: (data) => ipcRenderer.invoke("save-file-as", data),
      writeFile: () => ipcRenderer.invoke("write-file"),
    });
    contextBridge.exposeInMainWorld('api', api)
  } catch (error) {
    console.error(error)
  }
} else {
  window.electron = electronAPI
  window.api = api
}
