// src/ace/modes/dreamc_mode.js
// Define el modo y las reglas de resaltado para DreamC (versión actualizada) usando ace.require

// --- IMPORTA EL OBJETO 'ace' PRINCIPAL ---
import ace from 'react-ace-builds/node_modules/ace-builds/src-noconflict/ace';
// -----------------------------------------

// --- USA ace.require PARA OBTENER DEPENDENCIAS ---
const TextHighlightRules = ace.require("ace/mode/text_highlight_rules").TextHighlightRules;
const TextMode = ace.require("ace/mode/text").Mode;
// const oop = ace.require("ace/lib/oop"); // No necesario al usar 'extends'
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
        const numberRe = "(?:\\+|-)?(?:(?:\\d+\\.?\\d*|\\.\\d+)(?:[eE](?:\\+|-)?\\d+)?i?|\\d+i)"; // Color 1
        const arithmeticOperatorRe = "[\\+\\-\\*\\/]"; // Color 5: Solo simples +, -, *, /
        const otherOperatorRe = "\\>\\=|\\<\\=|==|!=|&&|\\|\\||[\\!\\<\\>]"; // Color 6: Relacionales y Lógicos (!, <, >)
        const assignmentOperatorRe = "="; // Color 6: Asignación simple
        // *** NUEVA REGLA *** para asignación compuesta (+=, -=, *=, /=)
        const compoundAssignmentRe = "[\\+\\-\\*\\/]="; // Color 6
        const stringRe = '"(?:[^\\"\\\\]|\\\\.)*"'; // Color 7
        const specialOperatorRe = "(?:@\\*|\\.\\.\\.\\+|\\|>|->)"; // Color 8


        this.$rules = {
            "start": [
                // Color 3: Comentarios
                { token: "comment.line.double-slash", regex: "\\/\\/.*$" },
                { token: "comment.block.documentation", regex: "\\/\\*", next: "comment" },

                // Color 7: Cadenas
                { token: "string.quoted.double", regex: stringRe },

                // Color 1: Números
                { token: "constant.numeric", regex: numberRe },

                // Color 8: Operadores Especiales
                { token: "keyword.operator.special", regex: specialOperatorRe },

                // *** NUEVO: *** Asignación Compuesta (antes que otros operadores)
                { token: "keyword.operator.assignment", regex: compoundAssignmentRe }, // Color 6

                // Color 6: Operadores Relacionales/Lógicos
                { token: "keyword.operator.logical", regex: otherOperatorRe },

                // Color 6: Asignación Simple
                { token: "keyword.operator.assignment", regex: assignmentOperatorRe },

                // Color 5: Operadores Aritméticos Simples
                { token: "keyword.operator.arithmetic", regex: arithmeticOperatorRe },

                // Color 4: Palabras Clave / Color 2: Identificadores
                { token: keywordMapper, regex: identifierRe },

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

