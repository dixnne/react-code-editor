import { app, shell, BrowserWindow, ipcMain, dialog } from 'electron';
import { join } from 'path';
import { electronApp, optimizer, is } from '@electron-toolkit/utils';
import fs from 'fs';
import icon from '../../resources/icon.png?asset';
import protoLoader from '@grpc/proto-loader';
import grpc from '@grpc/grpc-js';

// --- INICIO DE LA SECCIÓN CORREGIDA ---

// Define paths for both proto files
const LEXER_PROTO_PATH = join(__dirname, '../../protos/lexer.proto');
const PARSER_PROTO_PATH = join(__dirname, '../../protos/parser.proto');

// Opciones de carga del proto
const protoOptions = {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
  includeDirs: [join(__dirname, '../../protos')] // Directorio para buscar `import`
};

// Carga AMBAS definiciones de proto en un solo paso
const packageDefinition = protoLoader.loadSync(
  [LEXER_PROTO_PATH, PARSER_PROTO_PATH],
  protoOptions
);


// --- LÍNEAS DE DEPURACIÓN MEJORADAS ---
// 1. Cargar el objeto gRPC
const loadedGrpcObject = grpc.loadPackageDefinition(packageDefinition);
// 2. Imprimir los nombres de los paquetes que gRPC encontró
console.log("Paquetes cargados por gRPC:", Object.keys(loadedGrpcObject));


// 3. Acceder al paquete 'compiler'
const compilerProto = loadedGrpcObject.compiler;

// 4. Añadir una verificación explícita para dar un error claro
if (!compilerProto) {
    console.error("ERROR FATAL: El paquete 'compiler' no se encontró en los archivos .proto cargados.");
    console.error("Esto suele ocurrir por un problema de caché o de configuración.");
    console.error("Asegúrese de que ambos archivos .proto comienzan con 'package compiler;' y que ha limpiado los directorios 'out' y 'target'.");
    console.error("Paquetes que sí se encontraron:", Object.keys(loadedGrpcObject));
    app.quit(); // Salir de la aplicación si no se puede cargar el proto
}


// Crea los clientes desde el paquete 'compiler' unificado
const clientLexer = new compilerProto.Lexer('localhost:50051', grpc.credentials.createInsecure());
const clientParser = new compilerProto.Parser('localhost:50051', grpc.credentials.createInsecure());

console.log("Servicios gRPC cargados desde el paquete 'compiler' y listos.");

// --- FIN DE LA SECCIÓN CORREGIDA ---


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
      enableRemoteModule: false,
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

app.whenReady().then(() => {
  electronApp.setAppUserModelId('com.electron');

  app.on('browser-window-created', (_, window) => {
    optimizer.watchWindowShortcuts(window);
  });

  ipcMain.on('ping', () => console.log('pong'));

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

// --- Manejadores de IPC (sin cambios) ---

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

ipcMain.handle('run-parser', async (_event, code) => {
  console.log("Received code for parsing:", code);
  return new Promise((resolve, reject) => {
    clientParser.ParseSource({ source: code }, (err, response) => {
      if (err) {
        console.error("gRPC Parser Error:", err);
        console.error("Error details:", {
          code: err.code,
          message: err.message,
          details: err.details
        });
        reject(err.message);
      } else {
        console.log("gRPC Parser Response:", response);
        resolve(response);
      }
    });
  });
});
