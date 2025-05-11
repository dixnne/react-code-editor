// src/ace/modes/dreamc_highlight_rules.js
// Defines the syntax highlighting rules for the DreamC language

ace.define("ace/mode/dreamc_highlight_rules", ["require", "exports", "module", "ace/lib/oop", "ace/mode/text_highlight_rules"], function(require, exports, module) {
    "use strict";
    
    var oop = require("../lib/oop");
    var TextHighlightRules = require("./text_highlight_rules").TextHighlightRules;
    
    var DreamCHighlightRules = function() {
    
        // Regular expressions for different token types
        var keywords = (
            "if|else|end|do|while|switch|case|int|float|main|cin|cout"
        );
    
        var builtinConstants = (
            // Add any built-in constants like true/false if DreamC has them
            ""
        );
    
        var builtinFunctions = (
            // Add any built-in functions if DreamC has them
            ""
        );
    
        var keywordMapper = this.createKeywordMapper({
            "keyword.control": keywords, // Color 4: Palabras reservadas
            "constant.language": builtinConstants,
            "support.function": builtinFunctions
        }, "identifier", true); // Default token if no match (case-insensitive)
    
        var identifierRe = "[a-zA-Z_][a-zA-Z0-9_]*"; // Color 2: Identificadores
        var numberRe = "(?:\\+|-)?(?:\\d+\\.?\\d*|\\.\\d+)(?:[eE](?:\\+|-)?\\d+)?"; // Color 1: Números (enteros, reales, con/sin signo, notación científica básica)
        var operatorRe = "\\+|\\-|\\*|\\/|%|\\^|\\+\\+|--"; // Color 5: Operadores aritméticos
        var relationalRe = "<|>|<=|>=|!=|=="; // Color 6: Operadores relacionales
        var logicalRe = "&&|\\|\\||!"; // Color 6: Operadores lógicos (Nota: not es solo '!')
        var assignmentRe = "="; // Asignación
    
        // Rules for different states (main state is "start")
        this.$rules = {
            "start" : [
                {
                    token : "comment.line", // Color 3: Comentario de una línea
                    regex : "\\/\\/.*$"
                },
                {
                    token : "comment.block.start", // Color 3: Inicio comentario multilínea
                    regex : "\\/\\*",
                    next : "comment" // Go to comment state
                },
                {
                    token : "string.quoted.double", // Strings (si tu lenguaje los tuviera)
                    regex : '".*?"'
                },
                {
                    token : "string.quoted.single", // Strings (si tu lenguaje los tuviera)
                    regex : "'.*?'"
                },
                {
                    token : "constant.numeric", // Color 1: Números
                    regex : numberRe
                },
                {
                    token : keywordMapper, // Intenta mapear a keywords (Color 4)
                    regex : "[a-zA-Z_][a-zA-Z0-9_]*" // Si no es keyword, será 'identifier' (Color 2 por defecto)
                },
                {
                    token : "keyword.operator", // Color 5: Operadores aritméticos
                    regex : operatorRe
                },
                {
                    token : "keyword.operator", // Color 6: Operadores relacionales
                    regex : relationalRe
                },
                {
                    token : "keyword.operator", // Color 6: Operadores lógicos
                    regex : logicalRe
                },
                 {
                    token : "keyword.operator", // Asignación
                    regex : assignmentRe
                },
                {
                    token : "paren.lparen", // Símbolo: (
                    regex : "\\("
                }, {
                    token : "paren.rparen", // Símbolo: )
                    regex : "\\)"
                }, {
                    token : "paren.lparen", // Símbolo: { (usando mismo token que paréntesis)
                    regex : "\\{"
                }, {
                    token : "paren.rparen", // Símbolo: } (usando mismo token que paréntesis)
                    regex : "\\}"
                }, {
                    token : "punctuation.operator", // Símbolo: ,
                    regex : ","
                }, {
                    token : "punctuation.operator", // Símbolo: ;
                    regex : ";"
                }, {
                    token : "text", // Cualquier otro caracter
                    regex : "\\s+" // Espacios en blanco
                }
            ],
            // State for multi-line comments
            "comment" : [
                {
                    token : "comment.block.end", // Color 3: Fin comentario multilínea
                    regex : "\\*\\/",
                    next : "start" // Return to start state
                }, {
                    defaultToken : "comment.block" // Todo dentro es comentario
                }
            ]
        };
    
        this.normalizeRules();
    };
    
    oop.inherits(DreamCHighlightRules, TextHighlightRules);
    
    exports.DreamCHighlightRules = DreamCHighlightRules;
    });
    