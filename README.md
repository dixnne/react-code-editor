## 1. Diseño e Implementación

### 1.1. Visión General

El analizador léxico (o lexer) es la primera fase del compilador DreamC. Su función principal es leer el código fuente como una secuencia de caracteres y convertirlo en una secuencia de tokens. Cada token representa una unidad léxica significativa del lenguaje, como una palabra clave, un identificador, un número, un operador o un símbolo de puntuación. Este proceso simplifica las fases posteriores del compilador, como el análisis sintáctico.

El sistema completo consta de un backend en Rust que realiza el análisis léxico y un frontend en Electron/React que proporciona la interfaz de usuario.

### 1.2. Especificación de Reconocimiento de Tokens

* **Números** (Color 1)
    * Enteros y reales (con punto decimal).
    * Pueden tener signo opcional (`+` o `-`) al inicio.
    * Soporta notación científica básica (ej. `1.2e-3`).
    * *Ejemplos:* `42`, `-10`, `3.1415`, `0.5`, `-1.2E+5`
* **Identificadores** (Color 2)
    * Compuestos por letras (a-z, A-Z), dígitos (0-9) y guion bajo (`_`).
    * Deben comenzar con una letra o guion bajo.
    * *Ejemplos:* `x`, `miVariable`, `_temp`, `calcular_total`
* **Comentarios** (Color 3)
    * Comentarios de una línea: Comienzan con `//` y continúan hasta el final de la línea.
    * Comentarios multilínea: Comienzan con `/*` y terminan con `*/`.
    * *Ejemplos:* `// Esto es un comentario`, `/* Este es\nun comentario\nde varias líneas */`
* **Palabras Reservadas** (Color 4)
    * `let`, `const`, `fn`, `if`, `else`, `while`, `struct`, `return`, `for`, `in`
* **Operadores Aritméticos** (Color 5)
    * Suma: `+`
    * Resta: `-`
    * Multiplicación: `*`
    * División: `/`
    * Incremento: `++`
    * Decremento: `--`
* **Operadores Relacionales, Lógicos y de Asignación** (Color 6)
    * Relacionales: `>`, `<`, `>=`, `<=`, `==`, `!=`, `<>`
    * Lógicos: `&&`, `||`, `!`
    * Asignación: `=`, `+=`, `-=`, `*=`, `/=` (Nota: el lexer actual tokeniza `+=` como `+` y luego `=`)
* **Cadenas de Texto** (Color 7)
    * Secuencias de caracteres encerradas entre comillas dobles (`"`).
    * Secuencias de caracteres encerradas entre comillas simples (`'`).
    * Soportan secuencias de escape básicas (ej. `\n`, `\t`, `\\`, `\"`, `\'`).
    * *Ejemplos:* `"Hola, mundo!"`, `'Caracter'`
* **Operadores Especiales** (Color 8)
    * `@*`
    * `...+`
    * `|>`
    * `->`
* **Delimitadores y Puntuación** (Sin color específico asignado por defecto, usarán el color base del tema o el de identificador)
    * Paréntesis: `(`, `)`
    * Llaves: `{`, `}`
    * Corchetes: `[`, `]`
    * Coma: `,`
    * Punto y coma: `;`
    * Dos puntos: `:`
    * Punto: `.`

### 1.3. Componentes Principales

#### Backend (Rust)

* **`TokenType` (enum):**
    * Propósito: Define todos los posibles tipos de tokens que el analizador puede reconocer en el lenguaje DreamC.
    * Implementación: Enumeración de Rust (`enum`) con variantes para cada tipo de token. Deriva `Debug`, `Clone`, `PartialEq`, y `Eq` para facilitar pruebas y uso.

* **`LexerToken` (struct):**
    * Propósito: Representa una instancia de un token reconocido.
    * Estructura: `token_type` (TokenType), `lexeme` (String), `line` (usize), `column` (usize).
    * Implementación: Estructura de Rust con un constructor `new()`. Deriva `Debug`, `Clone`, y `PartialEq`.

* **`LexicalAnalyzer` (struct):**
    * Propósito: Realiza el proceso de escaneo del código fuente.
    * Campos: `input` (Peekable<Chars<'a>>), `line` (usize), `column` (usize), `keywords` (HashMap<String, TokenType>).
    * Implementación: Contiene métodos como `new()`, `advance()`, `peek()`, `match_next()`, `scan_tokens()`, y `scan_token()` para procesar la entrada carácter por carácter.

* **`LexerService` (struct gRPC):**
    * Propósito: Expone la funcionalidad del `LexicalAnalyzer` a través de una interfaz gRPC.
    * Implementación: Implementa el trait `Lexer`. Su método `analyze` utiliza `LexicalAnalyzer` para tokenizar el código de entrada.

* **`main()` (Función principal de Rust):**
    * Propósito: Configura y ejecuta el servidor gRPC (`tonic`) que aloja el `LexerService`.

#### Frontend (Electron/React)

* **`dreamc_mode.js` (Modo Ace Editor):**
    * Propósito: Define las reglas de resaltado de sintaxis para DreamC en Ace Editor.
    * Implementación: Clases `DreamCHighlightRules` y `DreamCMode`.

* **`TokenTable.jsx` (Componente React):**
    * Propósito: Muestra la lista de tokens en una tabla.
    * Implementación: Componente funcional de React que renderiza tokens en una tabla HTML estilizada.

* **`CodeEditor.jsx` (Componente React):**
    * Propósito: Integra `AceEditor`, configura el modo `dreamc`, y maneja la comunicación con el backend.

* **`grpcClient.js` (Cliente gRPC en Electron Main):**
    * Propósito: Conecta Electron con el servidor gRPC de Rust.

### 1.4. Algoritmo Principal (Análisis Léxico en Rust)

El `LexicalAnalyzer` utiliza un método `scan_token()` que se llama repetidamente. Este método consume el carácter actual y, mediante una serie de `match` y `if/else if`, determina el tipo de token. Utiliza `peek()` para observar el siguiente carácter sin consumirlo y `match_next()` para verificar y consumir un carácter esperado (para tokens de múltiples caracteres como `==` o `//`). Los identificadores se comparan con un `HashMap` de palabras clave para distinguirlos. Los números, strings y comentarios tienen bucles para consumir todos los caracteres que les pertenecen.

### 1.5. Integración gRPC

El `LexerService` recibe el código fuente, lo pasa al `LexicalAnalyzer`, y los tokens resultantes (después de filtrar algunos como `Whitespace` y `NewLine`) se envían de vuelta al cliente gRPC en el frontend.

## 2. Compilación y Ejecución

### 2.1. Requisitos Previos

* **Toolchain de Rust:** <https://rustup.rs/>
* **Compilador de Protocol Buffers (`protoc`):** (Opcional) <https://grpc.io/docs/protoc-installation/>
* **Dependencias de Rust:** Ver `backend/compiler/Cargo.toml`.
* **Node.js y npm/yarn:** Para Electron/React.

### 2.2. Estructura del Proyecto

```
react-code-editor/
├── backend/compiler/
│   ├── proto/
│   ├── src/
│   │   └── main.rs
│   ├── Cargo.toml
│   └── ...
├── src/ # Electron Main
│   └── ...
├── renderer/ # React
│   └── src/
│       ├── ace/
│       ├── components/
│       └── ...
└── package.json
```

### 2.3. Compilación

1.  **Compilar el Backend (Rust):**
    * Navega a `backend/compiler/`.
    * Ejecuta:
      ```bash
      cargo build
      ```
      (O `cargo build --release` para producción).

2.  **Instalar Dependencias del Frontend:**
    * Navega a la raíz del proyecto (`react-code-editor/`).
    * Ejecuta:
      ```bash
      npm install
      ```
      (O `yarn install`).

3.  **Compilar/Empaquetar el Frontend (Electron/React):**
    * Desde la raíz del proyecto, usa el script de tu `package.json`:
      ```bash
      npm run make # O el comando que uses (ej. build, package)
      ```

### 2.4. Ejecución (Modo Desarrollo)

1.  **Iniciar el Servidor Backend (Rust):**
    * En una terminal, navega a `backend/compiler/`.
    * Ejecuta:
      ```bash
      cargo run
      ```
    * Verifica que escuche en `[::1]:50051`.

2.  **Iniciar la Aplicación Frontend (Electron):**
    * En **otra** terminal, navega a la raíz (`react-code-editor/`).
    * Ejecuta:
      ```bash
      npm run start # O el comando de desarrollo que uses (ej. dev)
      ```

## 3. Pruebas y Validación del Analizador Léxico (Rust)

### 3.1. Estrategia de Pruebas

El analizador léxico en Rust se prueba mediante **pruebas unitarias** integradas con Cargo, ubicadas dentro de un módulo `#[cfg(test)] mod tests { ... }` en el archivo `backend/compiler/src/main.rs`.

Cada función de prueba:
1. Define una cadena de entrada (`input`) con código DreamC.
2. Crea una instancia de `LexicalAnalyzer` con esa entrada.
3. Llama a `analyzer.scan_tokens()` para obtener el vector de `LexerToken` resultante.
4. Define un vector `expected_tokens` con la secuencia exacta de tokens (tipo, lexema, línea, columna) que se espera para esa entrada, incluyendo el token `EndOfFile` al final.
5. Utiliza `assert_eq!(tokens_reales, tokens_esperados)` para comparar. Para las pruebas que no necesitan verificar `Whitespace` o `NewLine`, los tokens se filtran antes de la comparación.

Para ejecutar las pruebas, navega a la carpeta `backend/compiler/` y ejecuta:
```bash
cargo test
```

### 3.2. Pruebas Implementadas

El conjunto de pruebas actual en `backend/compiler/src/main.rs` incluye:

* **`test_simple_declaration()`:** Verifica la tokenización de una declaración de variable simple como `let x = 10;`. Se espera: `Keyword("let")`, `Identifier("x")`, `Equal("=")`, `Integer("10")`, `Semicolon(";")`, `EndOfFile`.
* **`test_operators()`:** Prueba una cadena que contiene una variedad de operadores aritméticos, relacionales, lógicos y especiales definidos en el lenguaje (ej. `+`, `-`, `==`, `!=`, `&&`, `++`, `->`, `@*`, `...+`, `|>`). Se verifica que cada uno se tokenice correctamente. Nota: `+=` se tokeniza como `Plus` y luego `Equal` según la lógica actual del lexer.
* **`test_comments()`:** Valida el reconocimiento de comentarios de una sola línea (`// ...`) y comentarios multilínea (`/* ... */`), incluyendo su contenido y la correcta continuación del análisis después de ellos. Esta prueba no filtra `NewLine` para asegurar la correcta posición de los tokens.
* **`test_comprehensive_code()`:** Utiliza un fragmento de código más extenso que combina declaraciones de variables, definiciones de funciones, estructuras condicionales (`if`), palabras clave, números, strings y comentarios para una validación más integral. Los tokens de `Whitespace`, `NewLine` y comentarios se filtran antes de la aserción.

### 3.3. Casos de Prueba Representativos

El siguiente fragmento de código DreamC intenta incluir la mayoría de los tipos de tokens especificados y puede usarse como base para una prueba unitaria o para validación manual en la interfaz:

```rust
// Ejemplo de Código DreamC para Pruebas

/* Comentario
   Multilínea */
let _identificador_123: int = -42; // Entero con signo
const VALOR_FLOTANTE: float = +123.45e-2;
let cadena_doble: string = "Hola \"mundo\" con\n\t escapes";
let cadena_simple: string = 'Caracteres simples';

fn mi_funcion(arg1: int, arg2: float) -> int {
    if arg1 >= 10 && arg2 < 0.5 || !flag {
        let resultado = arg1 * 5 / 2;
        resultado += 1; // Asignación compuesta
        resultado++;    // Incremento
    } else {
        return -1;
    }
    
    let arr = [1, 2, 3];
    let obj = {campo: 'valor'};
    
    // Operadores especiales
    let a = @*arr;
    let b = obj ...+ {otro: 1};
    let c = a |> funcion_pipe;
    
    return 0;
}

struct MiStruct {
    x: int,
    y: float,
}

// Bucle for (si se implementa)
// for item in lista { ... }

let x = 10;
while x > 0 {
    x--; // Decremento
}

// Llamada a función con flecha
let res = mi_funcion(5, 0.1) -> procesar;

// Chequeos finales
if x == 0 && y != 1 {
    // ok
}

// Token inválido (ejemplo)
$

```

**Validación Manual:**

1.  Pega el código anterior (o fragmentos específicos) en el editor de la aplicación Electron.
2.  Observa el resaltado de sintaxis. ¿Coincide con los colores esperados para cada token según la especificación?
3.  Ejecuta el análisis léxico desde la interfaz.
4.  Compara la tabla de tokens generada con la secuencia esperada manualmente para ese fragmento de código. Verifica `tokenType`, `lexeme`, `line` y `column`.

Combinando pruebas unitarias automatizadas en Rust y validación manual a través de la interfaz, puedes asegurar un alto grado de confianza en el correcto funcionamiento de tu analizador léxico.

