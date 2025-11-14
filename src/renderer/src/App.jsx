import React, { useState, useEffect, useRef } from "react";
import AceEditor from "react-ace";
import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import {
    FilePlus, Folder, Save, Printer, FileX, Play, Palette, Menu, FolderOpen, X
} from "lucide-react";
import clsx from "clsx";

// Importaciones de ACE Editor
import "react-ace-builds/node_modules/ace-builds/src-noconflict/theme-dracula";
import "react-ace-builds/node_modules/ace-builds/src-noconflict/ext-language_tools";
import "react-ace-builds/node_modules/ace-builds/src-noconflict/mode-javascript";
import DreamCMode from "./ace/mode/dreamc_mode";
import Themes from "./assets/themes.js";

// Simula la API de Electron si no está disponible
if (!window.electron) {
    window.electron = {
        openFile: async () => ({ content: "// Archivo de ejemplo\nfunction main() {}", path: "/ruta/falsa/ejemplo.dc" }),
        saveFile: async (file) => true,
        saveFileAs: async (file) => ({ path: "/ruta/falsa/nuevo.dc" }),
        runLexer: async (code) => {
            console.log("Mock runLexer");
            if (code.includes("error")) return { tokens: [{ token_type: "Invalid", lexeme: "error", line: 1, column: 1 }] };
            return { tokens: [{ token_type: "Keyword", lexeme: "function", line: 1, column: 1 }, { token_type: "Identifier", lexeme: "main", line: 1, column: 10 }] };
        },
        runCompiler: async (code) => {
            console.log("Mock runCompiler con nueva estructura");
            if (code.includes("error")) {
                return {
                    lexer_response: { tokens: [{ token_type: "Keyword", lexeme: "function", line: 1, column: 1 }] },
                    parse_response: { ast: null, errors: [{ message: "Token inesperado", line: 1, column: 10 }] },
                    semantic_response: { errors: [{ message: "Variable no declarada", line: 2, column: 5 }] }
                };
            }
            return {
                lexer_response: { tokens: [{ token_type: "Keyword", lexeme: "function", line: 1, column: 1 }, { token_type: "Identifier", lexeme: "main", line: 1, column: 10 }] },
                parse_response: { ast: { node_type: "Program", children: [{ node_type: "FunctionDeclaration", value: "main" }] }, errors: [] },
                semantic_response: { 
                    annotated_ast: { node_type: "Program", children: [{ node_type: "FunctionDeclaration", value: "main", inferred_type: "void" }], inferred_type: "void" }, 
                    errors: [], 
                    symbol_table: {
                        root_scope: {
                            symbols: { 'myGlobal': { type: 'int', defined_at: { line: 1, column: 5 } } },
                            children_scopes: [
                                {
                                    scope_name: "main",
                                    symbols: { 'x': { type: 'float', defined_at: { line: 3, column: 10 } } },
                                    children_scopes: []
                                }
                            ]
                        }
                    } 
                }
            };
        },
        llvmTranslate: async (code) => ({ llvm_ir: "; LLVM IR mock\ndefine i32 @main() {\nentry:\n  ret i32 0\n}" }),
        llvmOptimize: async (code) => ({ optimized_ir: "; Optimized LLVM IR mock\ndefine i32 @main() {\nentry:\n  ret i32 0\n}" }),
        executeProgram: async (code) => ({ exit_code: 0, output: "Program executed successfully", error: "" }),
    };
}


// --- Componentes ---

const TokenTable = ({ tokens, emptyMessage = 'No hay tokens para mostrar.' }) => {
    const validTokens = tokens?.tokens?.filter(token =>
        token.token_type !== "Invalid" && token.tokenType !== "CommentSingle" && token.tokenType !== "CommentMultiLine" && token.tokenType !== "NewLine"
    ) || [];

    if (validTokens.length === 0) {
        return <div className="p-4 text-center text-gray-400">{emptyMessage}</div>;
    }

    return (
        <div className="h-full overflow-auto">
            <table className="w-full text-sm text-left text-gray-300">
                <thead className="text-xs text-gray-300 uppercase bg-gray-700 sticky top-0">
                    <tr>
                        <th scope="col" className="px-6 py-3">Tipo de Token</th>
                        <th scope="col" className="px-6 py-3">Lexema</th>
                        <th scope="col" className="px-6 py-3">Línea</th>
                        <th scope="col" className="px-6 py-3">Columna</th>
                    </tr>
                </thead>
                <tbody className="divide-y divide-gray-700">
                    {validTokens.map((token, index) => (
                        <tr key={index} className="hover:bg-gray-600">
                            <td className="px-6 py-2 font-medium whitespace-nowrap text-white">{token.token_type}</td>
                            <td className="px-6 py-2">"{token.lexeme}"</td>
                            <td className="px-6 py-2">{token.line}</td>
                            <td className="px-6 py-2">{token.column}</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
};

const TreeNode = ({ node, theme }) => {
    const [isExpanded, setIsExpanded] = useState(true);
    const hasChildren = node.children && node.children.length > 0;

    const nodeTypeColor = theme.tertiary;
    const nodeValueColor = theme.syntax.keyword;
    const locationColor = theme.text;
    const hoverBgColor = theme.secondarySemi;
    const borderColor = theme.text;
    const inferredTypeColor = theme.syntax.string; 

    return (
        <div>
            <div
                className="flex items-center p-0.5 rounded cursor-pointer"
                onClick={() => hasChildren && setIsExpanded(!isExpanded)}
                style={{ '--hover-bg': hoverBgColor }}
                onMouseOver={e => e.currentTarget.style.backgroundColor = `var(--hover-bg)`}
                onMouseOut={e => e.currentTarget.style.backgroundColor = 'transparent'}
            >
                <span className="w-4 text-center text-xs" style={{ color: locationColor }}>
                    {hasChildren ? (isExpanded ? '▼' : '►') : '•'}
                </span>
                <div className="flex items-baseline gap-x-2">
                    <span className="font-semibold" style={{ color: nodeTypeColor }}>{node.node_type}</span>
                    {node.value && <span style={{ color: nodeValueColor }}>: {node.value}</span>}
                    {node.inferred_type && <span className="italic" style={{ color: inferredTypeColor }}> → {node.inferred_type}</span>}
                    {(node.start_line !== undefined) && <span className="text-xs font-bold" style={{ color: locationColor }}>(L{node.start_line}:{node.start_column})</span>}
                </div>
            </div>
            {isExpanded && hasChildren && (
                <div
                    className="ml-2 pl-2"
                    style={{ borderLeft: `1px dashed ${borderColor}` }}
                >
                    {node.children.map((child, index) => <TreeNode key={index} node={child} theme={theme} />)}
                </div>
            )}
        </div>
    );
};

const TreeView = ({ data, theme }) => {
    if (!data || !data.ast) {
        return <div className="p-4 text-center" style={{color: theme.tertiary + '80'}}>
            { 'AST no disponible. Verifique errores o espere la compilación.' }
        </div>;
    }
    return (
        <div className="font-mono text-sm p-2 h-full overflow-auto" style={{ color: theme.tertiary }}>
            <TreeNode node={data.ast} theme={theme} />
        </div>
    );
};

// --- MODIFICADO ---: Añadida la columna "Valor" para mostrar el valor de las variables.
const SymbolTableView = ({ symbolTable, theme }) => {
    if (!symbolTable || !symbolTable.root_scope) {
        return <div className="p-4 text-center text-gray-400">Tabla de símbolos no disponible.</div>;
    }

    const Scope = ({ scope, name, level = 0 }) => {
        const validSymbols = Array.isArray(scope.symbols) ? scope.symbols.filter(Boolean) : [];
        const hasSymbols = validSymbols.length > 0;

        return (
            <div className="font-mono text-sm" style={{ paddingLeft: `${level * 20}px` }}>
                <div className="font-bold py-1" style={{ color: theme.tertiary }}>
                    Ámbito: <span style={{ color: theme.syntax.keyword }}>{name}</span>
                </div>
                {hasSymbols ? (
                    <table className="w-full text-left text-xs my-2">
                        <thead style={{ color: theme.tertiary + '99' }}>
                            <tr>
                                <th className="px-2 py-1 border-b" style={{ borderColor: theme.secondary }}>Identificador</th>
                                <th className="px-2 py-1 border-b" style={{ borderColor: theme.secondary }}>Tipo Símbolo</th>
                                <th className="px-2 py-1 border-b" style={{ borderColor: theme.secondary }}>Tipo Dato</th>
                                <th className="px-2 py-1 border-b" style={{ borderColor: theme.secondary }}>Definido en</th>
                                {/* --- NUEVO ---: Cabecera para la columna de valor */}
                                <th className="px-2 py-1 border-b" style={{ borderColor: theme.secondary }}>Valor</th>
                            </tr>
                        </thead>
                        <tbody style={{ color: theme.tertiary }}>
                            {validSymbols.map((symbol) => (
                                <tr key={symbol.name + symbol.line + symbol.column}>
                                    <td className="px-2 py-1 border-b" style={{ borderColor: theme.secondary, color: theme.syntax.identifier }}>{symbol.name}</td>
                                    <td className="px-2 py-1 border-b" style={{ borderColor: theme.secondary, color: theme.syntax.string }}>{symbol.symbol_type}</td>
                                    <td className="px-2 py-1 border-b" style={{ borderColor: theme.secondary, color: theme.syntax.numeric }}>{symbol.data_type}</td>
                                    <td className="px-2 py-1 border-b" style={{ borderColor: theme.secondary }}>L{symbol.line}:{symbol.column}</td>
                                    {/* --- NUEVO ---: Celda para mostrar el valor de la variable */}
                                    <td className="px-2 py-1 border-b" style={{ borderColor: theme.secondary }}>
                                        {symbol.symbol_type === 'Variable' ? `"${symbol.value}"` : '—'}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                ) : (
                    <p className="text-xs text-gray-500 px-2 italic"> (No hay símbolos en este ámbito)</p>
                )}
                
                {Array.isArray(scope.children) && scope.children.map((childScope, index) => (
                    <Scope key={index} scope={childScope} name={childScope.scope_name || `hijo_${index}`} level={level + 1} />
                ))}
            </div>
        );
    };

    return (
        <div className="p-3 overflow-auto h-full">
            <Scope scope={symbolTable.root_scope} name={symbolTable.root_scope.scope_name || 'Global'} />
        </div>
    );
};

const Sidebar = ({ isOpen, onClose }) => (
    <div className={clsx("fixed inset-y-0 left-0 z-40 w-64 bg-gray-800 transform transition-transform duration-300", { "translate-x-0": isOpen, "-translate-x-full": !isOpen })}>
        <div className="flex items-center justify-between p-4 border-b border-gray-700">
            <h5 className="text-lg font-semibold text-gray-200">Explorador</h5>
            <button onClick={onClose} className="p-1 rounded-md hover:bg-gray-700">
                <X size={20} className="text-gray-400" />
            </button>
        </div>
        <div className="p-4 text-gray-300"><p>Contenido del explorador...</p></div>
    </div>
);

const NavDropdown = ({ icon, items, align = 'left' }) => {
    const [isOpen, setIsOpen] = useState(false);
    const dropdownRef = useRef(null);

    useEffect(() => {
        const handleClickOutside = (event) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target)) setIsOpen(false);
        };
        document.addEventListener("mousedown", handleClickOutside);
        return () => document.removeEventListener("mousedown", handleClickOutside);
    }, []);

    const menuClasses = clsx(
        "absolute mt-2 w-48 origin-top-left bg-gray-800 rounded-md shadow-lg ring-1 ring-black ring-opacity-5 z-20",
        { 'left-0': align === 'left', 'right-0': align === 'right' }
    );

    return (
        <div className="relative" ref={dropdownRef}>
            <button onClick={() => setIsOpen(!isOpen)} className="p-2 text-gray-300 bg-transparent rounded-md hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-900 focus:ring-white">
                {icon}
            </button>
            {isOpen && (
                <div className={menuClasses} role="menu" aria-orientation="vertical">
                    <div className="py-1">
                        {items.map((item) => (
                            <button key={item.label} onClick={() => { item.action(); setIsOpen(false); }}
                                className="w-full text-left px-4 py-2 text-sm text-gray-200 hover:bg-gray-700" role="menuitem">
                                {item.label}
                            </button>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
};


const IdeNav = ({ onAction, theme, changeTheme }) => {
    const [isSidebarOpen, setSidebarOpen] = useState(false);
    const handleAction = (action) => {
        onAction(null);
        setTimeout(() => onAction(action), 0);
    };

    const fileActions = [
        { icon: <FilePlus size={20} />, label: "New File", action: () => handleAction("new-file") },
        { icon: <FolderOpen size={20} />, label: "Open File", action: () => handleAction("open-file") },
        { icon: <Save size={20} />, label: "Save", action: () => handleAction("save-file") },
        { icon: <Printer size={20} />, label: "Save as", action: () => handleAction("save-file-as") },
        { icon: <FileX size={20} />, label: "Close", action: () => handleAction("close-file") },
    ];
    
    const themeItems = Object.values(Themes).map(th => ({
        label: th.name,
        action: () => changeTheme(th.code)
    }));

    return (
        <>
            <nav className="flex items-center justify-between gap-4 p-2 text-white" style={{ backgroundColor: theme.primary }}>
                <div className="flex items-center gap-2">
                    <button onClick={() => setSidebarOpen(true)} className="p-2 text-gray-300 rounded-md hover:bg-gray-700" title="Open Explorer"><Menu size={20} /></button>
                    <NavDropdown icon={<Folder size={20} />} items={[{ label: "Open Folder", action: () => handleAction("open-folder") }, { label: "Save File", action: () => handleAction("save-file") }, { label: "Close Folder", action: () => handleAction("close-folder") }]} />
                </div>
                <div className="flex items-center gap-2">
                    {fileActions.map((item, index) => <button key={index} onClick={item.action} title={item.label} className="p-2 text-gray-300 rounded-md hover:bg-gray-700">{item.icon}</button>)}
                </div>
                <div className="flex items-center gap-2">
                     <NavDropdown align="right" icon={<Play size={20} />} items={[{ label: "Run", action: () => handleAction("run") }, { label: "Debug", action: () => handleAction("debug") }]} />
                     <NavDropdown align="right" icon={<Palette size={20} />} items={themeItems} />
                </div>
            </nav>
            <Sidebar isOpen={isSidebarOpen} onClose={() => setSidebarOpen(false)} />
            {isSidebarOpen && <div className="fixed inset-0 bg-black opacity-50 z-30" onClick={() => setSidebarOpen(false)}></div>}
        </>
    );
};

const Tabs = ({ items, activeTab, setActiveTab, theme }) => (
    <div 
    // Add these two classes:
    className="flex items-center border-b px-2 overflow-x-auto whitespace-nowrap min-h-9 h-9" 
    style={{ borderColor: theme.primary, backgroundColor: theme.secondary }}
    >
    {items.map((item, index) => (
        <button 
        key={item} 
        onClick={() => setActiveTab(index)}
        className={clsx(
            "px-4 py-2 -mb-px text-sm font-medium border-b-2 transition-colors duration-200", 
            { 
            "border-b-white": activeTab === index, 
            // Typo fixed here (was activeTop)
            "border-transparent": activeTab !== index 
            }
        )}
        style={{
            color: theme.secondary,
            ...(activeTab === index && {color: theme.primary, backgroundColor: theme.tertiary}),
            ...(activeTab !== index && {color: theme.text})
        }}
        >
        {item}
        </button>
    ))}
    </div>
);

const DynamicAceStyles = ({ theme }) => {
    useEffect(() => {
        const styleId = 'dynamic-ace-theme-styles';
        document.getElementById(styleId)?.remove();
        if (!theme.syntax) return;

        const style = document.createElement('style');
        style.id = styleId;
        const s = theme.syntax;

        style.innerHTML = `
            .ace_editor .ace_constant.ace_numeric { color: ${s.numeric} !important; }
            .ace_editor .ace_identifier, .ace_editor .ace_paren, .ace_editor .ace_punctuation { color: ${s.identifier} !important; }
            .ace_editor .ace_comment { color: ${s.comment} !important; font-style: italic; }
            .ace_editor .ace_keyword { color: ${s.keyword} !important; font-weight: bold; }
            .ace_editor .ace_keyword.ace_operator[title="arithmetic"] { color: ${s.operatorArithmetic} !important; }
            .ace_editor .ace_keyword.ace_operator { color: ${s.operatorLogical} !important; }
            .ace_editor .ace_string { color: ${s.string} !important; }
            .ace_editor .ace_keyword.ace_operator[title="special"] { color: ${s.operatorSpecial} !important; }
        `;
        document.head.appendChild(style);
        return () => {
            document.getElementById(styleId)?.remove();
        };
    }, [theme]);

    return null;
};


// --- Componente Principal de la App ---
function App() {
    const [action, setAction] = useState(null);
    const [themeCode, setThemeCode] = useState("pointerOfDoom");
    const [editorContent, setEditorContent] = useState("// ¡Bienvenido a Pointer of Doom!\n");
    const [filePath, setFilePath] = useState(null);
    const [line, setLine] = useState(1);
    const [column, setColumn] = useState(1);
    const [analysisTab, setAnalysisTab] = useState(0);
    const [consoleTab, setConsoleTab] = useState(0);

    const [tokens, setTokens] = useState({ tokens: [] });
    const [syntax, setSyntax] = useState({ ast: null, errors: [] });
    // --- MODIFICADO ---: El estado semántico ahora incluye la tabla de símbolos
    const [semantic, setSemantic] = useState({ annotated_ast: null, errors: [], symbol_table: null }); 
    const [llvmIR, setLlvmIR] = useState("");
    const [optimizedIR, setOptimizedIR] = useState("");
    const [executionResult, setExecutionResult] = useState({ exit_code: null, output: "", error: "" }); 

    const aceEditorRef = useRef(null);
    const currentTheme = Themes[themeCode] || Themes.pointerOfDoom;

    useEffect(() => {
        if (aceEditorRef.current) {
            const editor = aceEditorRef.current.editor;
            const session = editor.getSession();
            const customMode = new DreamCMode();
            session.setMode(customMode);
        }
    }, []);

    useEffect(() => {
        const runAction = async () => {
            if (!action) return;
            const electronAPI = window.electron;
            try {
                switch (action) {
                    case "open-file": { const r = await electronAPI.openFile(); if(r?.content !== undefined) { setEditorContent(r.content); setFilePath(r.path); } break; }
                    case "save-file": { if (!filePath) { handleAction("save-file-as"); } else { await electronAPI.saveFile({ path: filePath, content: editorContent }); } break; }
                    case "save-file-as": { const r = await electronAPI.saveFileAs({ content: editorContent }); if (r?.path) setFilePath(r.path); break; }
                    case "new-file": setEditorContent("// Nuevo archivo DreamC\n"); setFilePath(null); break;
                    case "close-file": setEditorContent(""); setFilePath(null); break;
                }
            } catch(e) { console.error("Error en acción de archivo:", e); }
            setAction(null);
        };
        runAction();
    }, [action, editorContent, filePath]);

    useEffect(() => {
        const compileCode = async () => {
            if (editorContent.trim() === "") {
                setTokens({ tokens: [] });
                setSyntax({ ast: null, errors: [] });
                setSemantic({ annotated_ast: null, errors: [], symbol_table: null });
                setLlvmIR("");
                setOptimizedIR("");
                setExecutionResult({ exit_code: null, output: "", error: "" });
                return;
            }
            try {
                const lexerResult = await window.electron.runLexer(editorContent);
                if (lexerResult) setTokens(lexerResult);
                
                const result = await window.electron.runCompiler(editorContent);
                if (result) {
                    setSyntax(result.parse_response || { ast: null, errors: [] });
                    setSemantic(result.semantic_response || { annotated_ast: null, errors: [], symbol_table: null });
                }

                // Only fetch LLVM IR and execute if there are no errors
                const hasErrors = (result?.parse_response?.errors?.length > 0) || 
                                  (result?.semantic_response?.errors?.length > 0);
                
                if (!hasErrors) {
                    try {
                        const llvmResult = await window.electron.llvmTranslate(editorContent);
                        if (llvmResult) setLlvmIR(llvmResult.llvm_ir || "");

                        const optimizedResult = await window.electron.llvmOptimize(editorContent);
                        if (optimizedResult) setOptimizedIR(optimizedResult.optimized_ir || "");

                        const execResult = await window.electron.executeProgram(editorContent);
                        if (execResult) setExecutionResult(execResult);
                    } catch (llvmError) {
                        console.error("Error in LLVM/execution phase:", llvmError);
                        setLlvmIR("");
                        setOptimizedIR("");
                        setExecutionResult({ exit_code: -1, output: "", error: String(llvmError) });
                    }
                } else {
                    setLlvmIR("");
                    setOptimizedIR("");
                    setExecutionResult({ exit_code: null, output: "", error: "" });
                }
            } catch (error) {
                console.error("Error durante la compilación:", error);
                setTokens({ tokens: [] });
                setSyntax({ ast: null, errors: [] });
                setSemantic({ annotated_ast: null, errors: [], symbol_table: null });
                setLlvmIR("");
                setOptimizedIR("");
                setExecutionResult({ exit_code: null, output: "", error: "" });
            }
        };
        const debounce = setTimeout(compileCode, 500);
        return () => clearTimeout(debounce);
    }, [editorContent]);
    
    const handleAction = (newAction) => setAction(newAction);

    // --- MODIFICADO ---: `renderAnalysisPanel` ahora muestra la tabla de símbolos en la pestaña "Hash"
    const renderAnalysisPanel = () => {
        switch (analysisTab) {
            case 0: return <TokenTable tokens={tokens} />;
            case 1: return <TreeView data={syntax} theme={currentTheme} />;
            case 2: return <TreeView data={{ ast: semantic.annotated_ast }} theme={currentTheme} />;
            case 3: return <div className="p-4 font-mono text-sm whitespace-pre-wrap overflow-auto h-full" style={{color: currentTheme.tertiary, backgroundColor: currentTheme.background + "50"}}>{llvmIR || "No LLVM IR available. Check for errors."}</div>;
            case 4: {
                // Check if optimization made changes
                if (!optimizedIR) {
                    return <div className="p-4 font-mono text-sm whitespace-pre-wrap overflow-auto h-full" style={{color: currentTheme.tertiary, backgroundColor: currentTheme.background + "50"}}>No optimization available. Check for errors.</div>;
                }
                if (llvmIR === optimizedIR) {
                    return <div className="p-4 font-mono text-sm whitespace-pre-wrap overflow-auto h-full" style={{color: currentTheme.tertiary, backgroundColor: currentTheme.background + "50"}}>No optimization applied (code already optimal)</div>;
                }
                return <div className="p-4 font-mono text-sm whitespace-pre-wrap overflow-auto h-full" style={{color: currentTheme.tertiary, backgroundColor: currentTheme.background + "50"}}>{optimizedIR}</div>;
            }
            // --- NUEVO ---: Llama al nuevo componente para renderizar la tabla de símbolos
            case 5: return <SymbolTableView symbolTable={semantic.symbol_table} theme={currentTheme} />;
            default: return null;
        }
    };
    
    const renderConsolePanel = () => {
        const ErrorsDisplay = ({ errors, type }) => (
            <div className="p-4 text-sm font-mono text-gray-300 overflow-auto h-full">
                {(!errors || errors.length === 0)
                    ? `No hay errores de tipo ${type}.`
                    : errors.map((err, i) => <p key={i} className="text-red-400">Error: {err.message || JSON.stringify(err)} en línea {err.line} col {err.column}</p>)
                }
            </div>
        );
        const ExecutionDisplay = () => {
            if (executionResult.exit_code === null) {
                return <div className="p-4 text-gray-400">No execution results. Compile code without errors first.</div>;
            }
            
            const hasOutput = executionResult.output && executionResult.output.trim().length > 0;
            const hasError = executionResult.error && executionResult.error.trim().length > 0;
            
            return (
                <div className="p-4 text-sm font-mono text-gray-300 overflow-auto h-full">
                    <div className="mb-2">
                        <span className="font-bold">Exit Code: </span>
                        <span className={executionResult.exit_code === 0 ? "text-green-400" : "text-red-400"}>
                            {executionResult.exit_code}
                        </span>
                    </div>
                    {hasOutput && (
                        <div className="mb-2">
                            <div className="font-bold text-blue-400">Output:</div>
                            <pre className="whitespace-pre-wrap">{executionResult.output}</pre>
                        </div>
                    )}
                    {hasError && (
                        <div>
                            <div className="font-bold text-red-400">Error:</div>
                            <pre className="whitespace-pre-wrap text-red-400">{executionResult.error}</pre>
                        </div>
                    )}
                    {!hasOutput && !hasError && executionResult.exit_code === 0 && (
                        <div className="text-gray-500 italic">Program completed with no output</div>
                    )}
                </div>
            );
        };
        
        switch (consoleTab) {
            case 0: return <ErrorsDisplay errors={tokens.tokens?.filter(t => t.token_type === "Invalid")} type="léxico" />;
            case 1: return <ErrorsDisplay errors={syntax.errors} type="sintáctico" />;
            case 2: return <ErrorsDisplay errors={semantic.errors} type="semántico" />;
            case 3: return <ExecutionDisplay />;
            default: return null;
        }
    };

    return (
        <div className="flex flex-col h-screen" style={{ backgroundColor: currentTheme.background }}>
            <DynamicAceStyles theme={currentTheme} />
            <IdeNav onAction={handleAction} theme={currentTheme} changeTheme={setThemeCode} />
            <div className="flex-grow p-4">
                <PanelGroup direction="vertical" className="h-full w-full border rounded-lg" style={{ borderColor: currentTheme.primary }}>
                    <Panel defaultSize={75} minSize={20}>
                        <PanelGroup direction="horizontal">
                            <Panel defaultSize={60} minSize={30}>
                                <div className="flex flex-col h-full w-full rounded-tl-md overflow-hidden" style={{backgroundColor: currentTheme.secondary}}>
                                    <AceEditor
                                        ref={aceEditorRef}
                                        width="100%"
                                        height="calc(100% - 2rem)"
                                        theme="dracula"
                                        onChange={setEditorContent}
                                        value={editorContent}
                                        name="DREAMC_EDITOR"
                                        editorProps={{ $blockScrolling: true }}
                                        fontSize={14}
                                        showPrintMargin={true}
                                        showGutter={true}
                                        highlightActiveLine={true}
                                        setOptions={{ useWorker: false, enableBasicAutocompletion: true, enableLiveAutocompletion: true, enableSnippets: true, showLineNumbers: true, tabSize: 4 }}
                                        onCursorChange={(selection) => { const pos = selection.getCursor(); setLine(pos.row + 1); setColumn(pos.column + 1); }}
                                    />
                                     <div className="flex justify-end items-center h-8 px-4 text-sm" style={{ backgroundColor: currentTheme.tertiary, color: currentTheme.primary }}>
                                        <span>Ln {line}, Col {column}</span>
                                    </div>
                                </div>
                            </Panel>
                            <PanelResizeHandle className="w-2 bg-gray-700 hover:bg-blue-600 transition-colors" />
                            <Panel defaultSize={40} minSize={20}>
                                <div className="flex flex-col h-full rounded-tr-md" style={{ backgroundColor: currentTheme.primary }}>
                                    <Tabs items={["Lexical", "Syntactic", "Semantic", "Intermediate", "Optimizations", "Hash"]} activeTab={analysisTab} setActiveTab={setAnalysisTab} theme={currentTheme} />
                                    <div className="flex-grow overflow-auto" style={{ backgroundColor: currentTheme.secondarySemi }}>{renderAnalysisPanel()}</div>
                                </div>
                            </Panel>
                        </PanelGroup>
                    </Panel>
                    <PanelResizeHandle className="h-2 bg-gray-700 hover:bg-blue-600 transition-colors" />
                    <Panel defaultSize={25} minSize={10}>
                         <div className="flex flex-col h-full" style={{ backgroundColor: currentTheme.primary }}>
                            <Tabs items={["Lexical Errors", "Syntax Errors", "Semantic Errors", "Execution"]} activeTab={consoleTab} setActiveTab={setConsoleTab} theme={currentTheme} />
                            <div className="flex-grow overflow-auto" style={{ backgroundColor: currentTheme.secondarySemi }}>{renderConsolePanel()}</div>
                            <div className="flex justify-end items-center h-8 px-4 text-sm" style={{ backgroundColor: currentTheme.tertiary, color: currentTheme.primary }}>
                                <span>{filePath ? filePath.split(/[\\/]/).pop() : "Untitled"} - DreamC main</span>
                            </div>
                        </div>
                    </Panel>
                </PanelGroup>
            </div>
        </div>
    );
}

export default App;