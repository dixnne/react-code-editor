# DreamC IDE & Compilador

Este proyecto es un Entorno de Desarrollo Integrado (IDE) y un compilador completo para el lenguaje de programación **DreamC**. Cuenta con una interfaz moderna y fácil de usar construida con Electron y React, y un potente compilador de backend escrito en Rust.

## Características

### IDE
- **Compilación en Tiempo Real**: El código se compila mientras escribes, proporcionando retroalimentación instantánea.
- **Resaltado de Sintaxis**: Resaltado de sintaxis personalizado para el lenguaje DreamC.
- **Interfaz Multi-panel**: Visualiza las diferentes etapas del proceso de compilación en tiempo real:
    - **Tokens**: Muestra la salida del analizador léxico.
    - **AST**: Visualiza el Árbol de Sintaxis Abstracta (AST) generado por el analizador sintáctico.
    - **AST Anotado**: Inspecciona el AST después del análisis semántico, con información de tipos.
    - **Tabla de Símbolos**: Explora la tabla de símbolos con ámbitos, variables y funciones.
    - **LLVM IR**: Examina la Representación Intermedia de LLVM generada.
    - **IR Optimizado**: Visualiza el IR de LLVM después de las optimizaciones.
- **Reporte de Errores**: Los errores de cada etapa de la compilación (léxico, sintáctico, semántico) se muestran claramente.
- **Gestión de Archivos**: Operaciones básicas de archivo como nuevo, abrir y guardar.
- **Temas**: Elige entre una variedad de temas de color para el editor.

### Compilador
- **Compilación Multi-etapa**: El compilador sigue un pipeline tradicional:
    1. Análisis Léxico
    2. Análisis Sintáctico
    3. Análisis Semántico
    4. Generación de IR de LLVM
    5. Optimización
    6. Generación de Ejecutable
- **Interfaz gRPC**: El compilador se ejecuta como un servidor gRPC, permitiendo una fácil integración con el frontend u otros clientes.
- **Interfaz de Línea de Comandos (CLI)**: También está disponible una CLI `dreamcc` para compilar archivos DreamC desde la terminal.

## El Lenguaje DreamC

DreamC es un lenguaje de tipado estático con una sintaxis similar a la de C. Soporta:

- **Variables y Constantes**: `let` para variables mutables y `const` para constantes inmutables.
- **Tipos de Datos**: `int`, `float`, `string`, `bool`.
- **Funciones**: Palabra clave `fn` para definir funciones con parámetros y valores de retorno tipados.
- **Estructuras**: `struct` para definir estructuras de datos personalizadas.
- **Flujo de Control**: Bucles `if/else`, `while`, `for`, y `do-until`.
- **Expresiones Ricas**: Operaciones binarias y unarias, llamadas a funciones, asignaciones y más.

### Sintaxis y Características en Detalle

#### Declaraciones
- **Variables**: `let identificador: tipo = valor;` (la inferencia de tipos está soportada si se omite `: tipo`).
- **Constantes**: `const IDENTIFICADOR: tipo = valor;`.
- **Funciones**: `fn nombre(param1: tipo, param2: tipo) -> tipo_retorno { ... }`.
- **Estructuras**: `struct NombreStruct { campo1: tipo, campo2: tipo }`.

#### Estructuras de Control
- **Condicionales**:
  ```dreamc
  if (condicion) {
      // ...
  } else if (otra_condicion) {
      // ...
  } else {
      // ...
  }
  ```
- **Bucles**:
  ```dreamc
  while (condicion) {
      // ...
  }

  do {
      // ...
  } until (condicion);

  for (elemento in iterable) {
      // ...
  }
  ```

#### Expresiones
- **Operadores Aritméticos**: `+`, `-`, `*`, `/`.
- **Operadores Relacionales**: `==`, `!=`, `<`, `>`, `<=`, `>=`.
- **Operadores Lógicos**: `&&` (y), `||` (o), `!` (no).
- **Llamadas a Funciones**: `nombre_funcion(arg1, arg2)`.
- **Acceso a Miembros**: `objeto.propiedad`.

**Ejemplo de Código:**
```dreamc
// Un programa simple en DreamC
fn main() -> int {
    let message = "¡Hola, DreamC!";
    print(message); // Asumiendo una función 'print' predefinida
    return 0;
}
```

## Primeros Pasos

### Prerrequisitos

- **Node.js y npm**: Para el frontend de Electron/React.
- **Toolchain de Rust**: Para el compilador de backend.
- **LLVM 18**: Requerido para compilar y enlazar el código generado.
- **GCC/Clang**: Para enlazar el ejecutable final.

### Modo de Desarrollo

1.  **Iniciar el Servidor del Compilador (Backend)**:
    ```bash
    cd backend/compiler
    cargo run
    ```
    El servidor se iniciará en `127.0.0.1:50051`.

2.  **Iniciar la Aplicación Frontend**:
    En una nueva terminal, desde la raíz del proyecto:
    ```bash
    npm install
    npm run dev
    ```

### Compilación para Producción

Para construir una aplicación distribuible, ejecuta el siguiente comando desde la raíz del proyecto:

```bash
npm run build
```

Esto creará una compilación lista para producción en el directorio `out/`. Luego puedes usar uno de los siguientes scripts para empaquetar la aplicación para tu plataforma:

- `npm run build:win`
- `npm run build:mac`
- `npm run build:linux`

## Arquitectura

La aplicación se divide en dos componentes principales:

1.  **Frontend**: Una aplicación Electron con una interfaz de usuario basada en React. Proporciona el editor de código y muestra los resultados de la compilación. Se comunica con el backend a través de gRPC.
2.  **Backend**: Una aplicación en Rust que contiene el compilador de DreamC. Se ejecuta como un servidor gRPC, exponiendo servicios para cada etapa del proceso de compilación (análisis léxico, sintáctico, semántico, etc.).

Esta arquitectura cliente-servidor desacopla la lógica de la interfaz de usuario de la del compilador, permitiendo que se desarrollen y mantengan de forma independiente.

## Funciones Importantes del Código

### Backend (Rust)

- **`dreamcc::compile` (`backend/compiler/src/dreamcc.rs`)**: Esta es la función principal del compilador CLI. Orquesta todo el proceso de compilación, desde la lectura del archivo de entrada hasta la generación del ejecutable final. Llama a cada etapa del compilador en secuencia (lexer, parser, analizador semántico, generador de IR de LLVM).

- **`LexicalAnalyzer::scan_tokens` (`backend/compiler/src/lexer.rs`)**: El corazón del analizador léxico. Recorre el código fuente de entrada carácter por carácter y lo convierte en una secuencia de `Tokens`.

- **`parse_tokens` (`backend/compiler/src/parser.rs`)**: Toma la lista de tokens del lexer y construye un Árbol de Sintaxis Abstracta (AST). También captura y reporta errores de sintaxis.

- **`SemanticAnalyzer::analyze` (`backend/compiler/src/semantic_analyzer.rs`)**: Recorre el AST para realizar el análisis semántico. Verifica cosas como declaraciones de variables, concordancia de tipos y el uso correcto de las funciones. Construye la tabla de símbolos.

- **`compile_to_llvm_ir` (`backend/compiler/src/llvm_compiler.rs`)**: Traduce el AST (verificado semánticamente) a la Representación Intermedia de LLVM (LLVM IR). Esta es la etapa final antes de la optimización y la generación de código de máquina.

- **`main` (`backend/compiler/src/main.rs`)**: El punto de entrada para el servidor gRPC. Inicia el servidor `tonic` y registra los servicios de gRPC (`Lexer`, `Parser`, `Compiler`) para que el frontend pueda llamarlos.

### Frontend (React)

- **`App.jsx`**: El componente raíz de la aplicación. Gestiona el estado general de la IDE, incluyendo el contenido del editor, los resultados del análisis y la interacción del usuario.

- **`useEffect` con `debounce` en `App.jsx`**: Este hook es crucial para la funcionalidad en tiempo real. Se activa cuando el contenido del editor cambia, y después de un breve retraso (debounce), llama a los servicios del compilador en el backend para obtener los nuevos resultados del análisis.

- **`grpcClient.js` (`src/main/grpcClient.js`)**: Configura el cliente gRPC que se comunica con el servidor de Rust. Define las funciones (ej. `runLexer`, `runCompiler`) que el frontend utiliza para enviar código al backend y recibir los resultados.

- **`renderAnalysisPanel` y `renderConsolePanel` en `App.jsx`**: Estas funciones renderizan dinámicamente los diferentes paneles de la interfaz de usuario (Tokens, AST, Errores, etc.) basados en la pestaña seleccionada por el usuario y los datos recibidos del backend.

## Desarrollo y Pruebas

### Ejecución de Pruebas

Para ejecutar las pruebas del compilador de backend, navega al directorio `backend/compiler` y ejecuta:

```bash
cargo test
```

Las pruebas utilizan `insta` para realizar "snapshot testing" del AST, comparando la salida del parser con una versión "correcta" guardada.

### Código de Pruebas

El directorio `backend/compiler/tests/` contiene las pruebas de integración.

**`integration_tests.rs`**:
```rust
// Usa el nombre del crate "compiler" para importar la lógica de la biblioteca
use compiler::lexer::LexicalAnalyzer;
use compiler::parser::parse_tokens;
use compiler::token::{LexerToken, TokenType};
use std::fs;
use std::path::Path;

fn run_snapshot_test(file_path: &str) {
    let path = Path::new(file_path);
    let source = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("No se pudo leer el archivo de prueba: {}", file_path));

    let mut lexer = LexicalAnalyzer::new(&source);
    let tokens = lexer.scan_tokens();

    let filtered_tokens: Vec<LexerToken> = tokens
        .into_iter()
        .filter(|t| !matches!(t.token_type, TokenType::Whitespace | TokenType::NewLine | TokenType::CommentSingle | TokenType::CommentMultiLine | TokenType::Unknown))
        .collect();
    
    let result = parse_tokens(&filtered_tokens);

    let ast_string = format!("{:#?}", result.ast);
    
    // El nombre del snapshot se deriva del nombre del archivo
    insta::assert_snapshot!(insta::internals::AutoName, ast_string, file_path);
}

#[test]
fn test_simple_let() {
    run_snapshot_test("tests/cases/01_simple_let.dreamc");
}

#[test]
fn test_functions() {
    run_snapshot_test("tests/cases/02_functions.dreamc");
}

#[test]
fn test_syntax_error() {
    run_snapshot_test("tests/cases/03_syntax_error.dreamc");
}
```

**Casos de Prueba (`backend/compiler/tests/cases/`)**:

`01_simple_let.dreamc`:
```dreamc
// tests/cases/01_simple_let.dreamc
let a: int = 10;
let b = a + 5;
```

`02_functions.dreamc`:
```dreamc
// tests/cases/02_functions.dreamc
fn add(a: int, b: int) -> int {
    return a + b;
}
```

`03_syntax_error.dreamc`:
```dreamc
// tests/cases/03_syntax_error.dreamc
let x: int = 10;
fn my_func() -> void {
    let y =; // <-- Error de sintaxis aquí
}
```

### Estructura del Proyecto
```
/
├── backend/compiler/   # Compilador de Rust y servidor gRPC
│   ├── src/
│   ├── proto/          # Definiciones de Protobuf para gRPC
│   └── Cargo.toml
├── src/                # Proceso principal de Electron
│   ├── main/
│   └── preload/
├── protos/             # Definiciones de Protobuf compartidas
├── resources/
└── src/renderer/       # Frontend de React (proceso de renderizado)
    └── src/
        ├── App.jsx
        ├── components/
        └── ...
```
