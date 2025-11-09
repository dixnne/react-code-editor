import { app, shell, BrowserWindow, ipcMain, dialog } from 'electron';
import { join } from 'path';
import { electronApp, optimizer, is } from '@electron-toolkit/utils';
import fs from 'fs';
import icon from '../../resources/icon.png?asset';
import protoLoader from '@grpc/proto-loader';
import grpc from '@grpc/grpc-js';

// --- UNIFIED PROTOBUF AND GRPC CLIENT SETUP ---

// Define the directory where your .proto files are located
const PROTO_DIR = join(__dirname, '../../protos');

// Define loading options
const protoOptions = {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
  includeDirs: [PROTO_DIR] // This is crucial for resolving imports within .proto files
};

// Load all proto definitions into a single package
const packageDefinition = protoLoader.loadSync(
  [
    join(PROTO_DIR, 'lexer.proto'),
    join(PROTO_DIR, 'parser.proto'),
    join(PROTO_DIR, 'semantic.proto')
  ],
  protoOptions
);

// Load the package into gRPC
const grpcObject = grpc.loadPackageDefinition(packageDefinition);
const compilerProto = grpcObject.compiler;

// Explicitly check if the compiler package was loaded correctly
if (!compilerProto) {
  console.error("FATAL ERROR: The 'compiler' package was not found in the loaded .proto files.");
  console.error("Please ensure your .proto files start with 'package compiler;'");
  app.quit();
}

// Create a SINGLE client for the main Compiler service
// This is more efficient than creating clients for each sub-service.
const clientLexer = new compilerProto.Lexer(
  'localhost:50051',
  grpc.credentials.createInsecure()
);

const clientParser = new compilerProto.Parser(
  'localhost:50051',
  grpc.credentials.createInsecure()
);

const clientCompiler = new compilerProto.Compiler(
  'localhost:50051',
  grpc.credentials.createInsecure()
);

// --- ELECTRON WINDOW CREATION ---

function createWindow() {
  const mainWindow = new BrowserWindow({
    width: 900,
    height: 670,
    show: false,
    autoHideMenuBar: true,
    ...(process.platform === 'linux' ? { icon } : {}),
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      sandbox: false,
      contextIsolation: true,
      nodeIntegration: false,
      enableRemoteModule: false
    }
  });

  mainWindow.on('ready-to-show', () => {
    mainWindow.show();
  });

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url);
    return { action: 'deny' };
  });

  if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
    mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL']);
  } else {
    mainWindow.loadFile(join(__dirname, '../renderer/index.html'));
  }
}

// --- ELECTRON APP LIFECYCLE ---

app.whenReady().then(() => {
  electronApp.setAppUserModelId('com.electron');

  app.on('browser-window-created', (_, window) => {
    optimizer.watchWindowShortcuts(window);
  });

  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// --- IPC HANDLERS ---

// A single handler for the entire compilation process
ipcMain.handle('run-compiler', async (_event, sourceCode) => {
  console.log("🚀 Received source code for compilation...");
  return new Promise((resolve, reject) => {
    // Call the 'Compile' RPC from your Compiler service
    clientCompiler.Compile({ source: sourceCode }, (err, response) => {
      if (err) {
        console.error("❌ gRPC Compiler Error:", err);
        reject(err.message);
      } else {
        console.log("✅ gRPC Compiler Response Received:", response);
        // The response contains everything: parse results, errors, and semantic analysis results.
        resolve(response);
      }
    });
  });
});

// Handler for LLVM intermediate code generation
ipcMain.handle('llvm-translate', async (_event, sourceCode) => {
  console.log("🔄 Generating LLVM IR...");
  return new Promise((resolve, reject) => {
    clientCompiler.LlvmTranslate({ source: sourceCode }, (err, response) => {
      if (err) {
        console.error("❌ gRPC LLVM Translate Error:", err);
        reject(err.message);
      } else {
        console.log("✅ LLVM IR Generated");
        resolve(response);
      }
    });
  });
});

// Handler for LLVM optimization
ipcMain.handle('llvm-optimize', async (_event, sourceCode) => {
  console.log("⚡ Optimizing LLVM IR...");
  return new Promise((resolve, reject) => {
    clientCompiler.LlvmOptimize({ source: sourceCode }, (err, response) => {
      if (err) {
        console.error("❌ gRPC LLVM Optimize Error:", err);
        reject(err.message);
      } else {
        console.log("✅ LLVM IR Optimized");
        resolve(response);
      }
    });
  });
});

// Handler for program execution
ipcMain.handle('execute-program', async (_event, sourceCode) => {
  console.log("▶️ Executing program...");
  return new Promise((resolve, reject) => {
    clientCompiler.Execute({ source: sourceCode }, (err, response) => {
      if (err) {
        console.error("❌ gRPC Execute Error:", err);
        reject(err.message);
      } else {
        console.log("✅ Program Executed");
        resolve(response);
      }
    });
  });
});

ipcMain.handle('run-lexer', async (_event, code) => {
  console.log("Received code for lexing:", code);
  return new Promise((resolve, reject) => {
    clientLexer.Analyze({ input: code }, (err, response) => {
      if (err) {
        console.error("gRPC Lexer Error:", err);
        reject(err.message);
      } else {
        console.log("gRPC Lexer Response:", response);
        resolve(response);
      }
    });
  });
});

// File I/O Handlers
ipcMain.handle("open-file", async () => {
  const { filePaths, canceled } = await dialog.showOpenDialog({
    title: "Select a File",
    buttonLabel: "Open",
    properties: ["openFile"],
    filters: [{ name: "All Files", extensions: ["*"] }]
  });

  if (!canceled && filePaths.length > 0) {
    const content = fs.readFileSync(filePaths[0], "utf-8");
    return { path: filePaths[0], content };
  }
  return null;
});

ipcMain.handle("open-folder", async () => {
  const { filePaths, canceled } = await dialog.showOpenDialog({
    title: "Select a Folder",
    buttonLabel: "Open",
    properties: ["openDirectory"]
  });

  return !canceled && filePaths.length > 0 ? filePaths[0] : null;
});

ipcMain.handle("save-file", async (_, { path, content }) => {
  if (path) {
    fs.writeFileSync(path, content, "utf-8");
    return true;
  }
  return false;
});

ipcMain.handle("write-file", async () => {
  const { filePath, canceled } = await dialog.showSaveDialog({
    title: "Save File",
    buttonLabel: "Save",
    filters: [{ name: "All Files", extensions: ["*"] }]
  });

  if (!canceled && filePath) {
    fs.writeFileSync(filePath, "", "utf-8");
    return filePath;
  }
  return null;
});

ipcMain.handle("save-file-as", async (event, data) => {
  const { filePath } = await dialog.showSaveDialog({
    title: "Save File",
    defaultPath: "untitled.c",
    filters: [{ name: "C Files", extensions: ["c"] }, { name: "All Files", extensions: ["*"] }]
  });

  if (filePath) {
    fs.writeFileSync(filePath, data.content, "utf-8");
    return { path: filePath };
  }

  return null;
});