// src/ace/modes/dreamc.js
// Defines the DreamC language mode for Ace Editor

ace.define("ace/mode/dreamc", ["require", "exports", "module", "ace/lib/oop", "ace/mode/text", "ace/mode/dreamc_highlight_rules"], function(require, exports, module) {
    "use strict";
    
    var oop = require("../lib/oop");
    var TextMode = require("./text").Mode;
    var DreamCHighlightRules = require("./dreamc_highlight_rules").DreamCHighlightRules;
    // var DreamCFoldMode = require("./folding/cstyle").FoldMode; // Opcional: si quieres folding estilo C
    
    var Mode = function() {
        this.HighlightRules = DreamCHighlightRules;
        // this.foldingRules = new DreamCFoldMode(); // Opcional: Habilitar folding
        this.$behaviour = this.$defaultBehaviour;
    };
    oop.inherits(Mode, TextMode);
    
    (function() {
        this.lineCommentStart = "//"; // Define el inicio de comentario de línea
        this.blockComment = {start: "/*", end: "*/"}; // Define comentarios de bloque
        // this.$indentation = { ... }; // Opcional: Reglas de indentación automática
        this.$id = "ace/mode/dreamc"; // Identificador único del modo
    }).call(Mode.prototype);
    
    exports.Mode = Mode;
    
    });
    