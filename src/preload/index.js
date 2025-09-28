const { contextBridge, ipcRenderer } = require('electron');

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
      runLexer: (code) => ipcRenderer.invoke("run-lexer", code),
      
      // --- CORRECCIÓN CRÍTICA ---
      // La función expuesta al frontend debe coincidir con el handler en `main/index.js`.
      // El handler se llama 'run-compiler', no 'compile'.
      runCompiler(data) { 
        return ipcRenderer.invoke("run-compiler", data);
      }
    });
    contextBridge.exposeInMainWorld('api', api)
  } catch (error) {
    console.error(error)
  }
} else {
  // Nota: Esto es para entornos sin contextIsolation, no debería ser tu caso.
  window.electron = {
    openFile: () => ipcRenderer.invoke("open-file"),
    openFolder: () => ipcRenderer.invoke("open-folder"),
    saveFile: (data) => ipcRenderer.invoke("save-file", data),
    saveFileAs: (data) => ipcRenderer.invoke("save-file-as", data),
    writeFile: () => ipcRenderer.invoke("write-file"),
    runLexer: (code) => ipcRenderer.invoke("run-lexer", code),
  }
  window.api = api
}
