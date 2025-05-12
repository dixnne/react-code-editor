// src/ace/modes/dreamc_mode.js
// Define el modo y las reglas de resaltado para DreamC (versión actualizada) usando ace.require

// --- IMPORTA EL OBJETO 'ace' PRINCIPAL ---
import ace from 'react-ace-builds/node_modules/ace-builds/src-noconflict/ace';
// -----------------------------------------

// --- USA ace.require PARA OBTENER DEPENDENCIAS ---
const TextHighlightRules = ace.require("ace/mode/text_highlight_rules").TextHighlightRules;
const TextMode = ace.require("ace/mode/text").Mode;
// ---------------------------------------------

// 1. Definición de las Reglas de Resaltado Actualizadas
export class DreamCHighlightRules extends TextHighlightRules {
    constructor() {
        super();

        const keywords = (
            "let|const|fn|if|else|while|struct|return|for|in"
        );
        const keywordMapper = this.createKeywordMapper({
            "keyword.control": keywords, // Color 4
        }, "identifier", true); // Color 2 por defecto

        const identifierRe = "[a-zA-Z_][a-zA-Z0-9_]*"; // Color 2
        const numberRe = "(?:\\+|-)?(?:\\d+\\.?\\d*|\\.\\d+)(?:[eE](?:\\+|-)?\\d+)?"; // Color 1
        // *** NUEVO: Regex para ++ y -- *** (Color 5)
        const incrementDecrementRe = "\\+\\+|--";
        const arithmeticOperatorRe = "[\\+\\-\\*\\/]"; // Color 5 (Solo simples)
        const otherOperatorRe = "\\>\\=|\\<\\=|==|!=|&&|\\|\\||[\\!\\<\\>]"; // Color 6
        const assignmentOperatorRe = "="; // Color 6
        const compoundAssignmentRe = "[\\+\\-\\*\\/]="; // Color 6
        const singleQuoteStringRe = "'(?:[^'\\\\]|\\\\.)*'"; // Color 7
        const doubleQuoteStringRe = '"(?:[^\\"\\\\]|\\\\.)*"'; // Color 7
        const specialOperatorRe = "(?:@\\*|\\.\\.\\.\\+|\\|>|->)"; // Color 8


        this.$rules = {
            "start": [
                // Color 3: Comentarios
                { token: "comment.line.double-slash", regex: "\\/\\/.*$" },
                { token: "comment.block.documentation", regex: "\\/\\*", next: "comment" },

                // Color 7: Cadenas
                { token: "string.quoted.single", regex: singleQuoteStringRe },
                { token: "string.quoted.double", regex: doubleQuoteStringRe },

                // Color 1: Números
                { token: "constant.numeric", regex: numberRe },

                // Color 8: Operadores Especiales
                { token: "keyword.operator.special", regex: specialOperatorRe },

                // Asignación Compuesta
                { token: "keyword.operator.assignment", regex: compoundAssignmentRe }, // Color 6

                // *** NUEVO: Regla para ++ y -- (antes de operadores simples) ***
                { token: "keyword.operator.arithmetic", regex: incrementDecrementRe }, // Color 5

                // Operadores Relacionales/Lógicos
                { token: "keyword.operator.logical", regex: otherOperatorRe }, // Color 6

                // Asignación Simple
                { token: "keyword.operator.assignment", regex: assignmentOperatorRe }, // Color 6

                // Operadores Aritméticos Simples (+, -, *, /)
                { token: "keyword.operator.arithmetic", regex: arithmeticOperatorRe }, // Color 5

                // Palabras Clave / Identificadores
                { token: keywordMapper, regex: identifierRe }, // Color 4 o 2

                // Delimitadores y Puntuación
                { token: "paren.lparen", regex: "[\\(]" },
                { token: "paren.rparen", regex: "[\\)]" },
                { token: "paren.lparen", regex: "[\\{]" },
                { token: "paren.rparen", regex: "[\\}]" },
                { token: "punctuation.operator", regex: "," },
                { token: "punctuation.operator", regex: ";" },

                // Espacio en blanco
                { token: "text", regex: "\\s+" }
            ],
            "comment": [
                { token: "comment.block.documentation", regex: "\\*\\/", next: "start" },
                { defaultToken: "comment.block.documentation" }
            ]
        };
        this.normalizeRules();
    }
}

// 2. Definición del Modo (Sin cambios)
export default class DreamCMode extends TextMode {
    constructor() {
        super();
        this.HighlightRules = DreamCHighlightRules;
        this.lineCommentStart = "//";
        this.blockComment = {start: "/*", end: "*/"};
    }
}


