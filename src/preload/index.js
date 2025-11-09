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
      runCompiler: (data) => ipcRenderer.invoke("run-compiler", data),
      llvmTranslate: (code) => ipcRenderer.invoke("llvm-translate", code),
      llvmOptimize: (code) => ipcRenderer.invoke("llvm-optimize", code),
      executeProgram: (code) => ipcRenderer.invoke("execute-program", code),
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
    runCompiler: (data) => ipcRenderer.invoke("run-compiler", data),
    llvmTranslate: (code) => ipcRenderer.invoke("llvm-translate", code),
    llvmOptimize: (code) => ipcRenderer.invoke("llvm-optimize", code),
    executeProgram: (code) => ipcRenderer.invoke("execute-program", code),
  }
  window.api = api
}
