import React, { useState, useEffect, useRef } from "react";
import AceEditor from "react-ace";
import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import {
    FilePlus, Folder, Save, Printer, FileX, Play, Palette, Menu, FolderOpen, X
} from "lucide-react";
import clsx from "clsx";

// Importaciones de ACE Editor. Se usa 'ace-builds' que es la dependencia estándar.
import "react-ace-builds/node_modules/ace-builds/src-noconflict/theme-dracula";
import "react-ace-builds/node_modules/ace-builds/src-noconflict/ext-language_tools";
import "react-ace-builds/node_modules/ace-builds/src-noconflict/mode-javascript"; // Modo por defecto, se reemplazará más adelante
import DreamCMode from "./ace/mode/dreamc_mode";

// --- FIX: Mocks for local files to resolve build errors in this environment ---
// In your local project, you should use your actual import statements.
// import DreamCMode from "./ace/mode/dreamc_mode.js";
import Themes from "./assets/themes.js";


// Simula la API de Electron si no está disponible (para desarrollo en navegador)
if (!window.electron) {
    window.electron = {
        openFile: async () => { console.log("Mock openFile called"); return { content: "// Archivo de ejemplo abierto\nfunction main() {\n  put('Hola, DreamC!');\n}", path: "/ruta/falsa/ejemplo.dc" }; },
        saveFile: async (file) => { console.log("Mock saveFile called with:", file); return true; },
        saveFileAs: async (file) => { console.log("Mock saveFileAs called with:", file); return { path: "/ruta/falsa/nuevo.dc" }; },
        runLexer: async (code) => {
            console.log("Mock runLexer");
            if (code.includes("error")) return { tokens: [{ token_type: "Invalid", lexeme: "error", line: 1, column: 1 }] };
            return { tokens: [{ token_type: "Keyword", lexeme: "function", line: 1, column: 1 }, { token_type: "Identifier", lexeme: "main", line: 1, column: 10 }] };
        },
        runParser: async (code) => {
            console.log("Mock runParser");
            if (code.includes("error")) return { ast: null, errors: [{ error_type: "Syntax Error", message: "Unexpected token", line: 1, column: 1 }] };
            return { ast: { node_type: "Program", children: [{ node_type: "FunctionDeclaration", value: "main" }] }, errors: [] };
        },
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

const TreeViewStyles = () => (
    <style>{`
        .tree-view-container { font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace; font-size: 14px; padding: 12px; color: #cbd5e1; }
        .tree-node .node-children { margin-left: 1rem; padding-left: 1rem; border-left: 1px dashed #4a5568; }
        .node-card { display: flex; align-items: center; padding: 4px; border-radius: 4px; cursor: pointer; transition: background-color 0.2s; }
        .node-card:hover { background-color: #374151; }
        .expand-icon { width: 20px; text-align: center; margin-right: 8px; color: #9ca3af; }
        .node-content { display: flex; align-items: center; gap: 0.5rem; }
        .node-type { font-weight: 600; color: #93c5fd; }
        .node-value { color: #a7f3d0; }
        .node-location { color: #6b7280; font-size: 12px; }
        .tree-empty-message { padding: 20px; text-align: center; color: #6b7280; }
    `}</style>
);

// TreeView Component - Themed and Compact
const TreeNode = ({ node, theme }) => {
    const [isExpanded, setIsExpanded] = useState(true);
    const hasChildren = node.children && node.children.length > 0;

    const isLightTheme = theme.code === 'light';
    const nodeTypeColor = theme.tertiary;
    const nodeValueColor = theme.text;
    const locationColor = theme.secondary;
    const hoverBgColor = theme.secondarySemi;
    const borderColor = theme.text;

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
                    {(node.start_line !== undefined) && <span className="text-xs" style={{ color: locationColor }}>(L{node.start_line}:{node.start_column})</span>}
                </div>
            </div>
            {isExpanded && hasChildren && (
                <div
                    className="ml-2 pl-2" // Compact indentation
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
            {!data ? 'No data provided' : 'AST no disponible. Verifique errores de sintaxis.'}
        </div>;
    }
    return (
        <div
            className="font-mono text-sm p-2 h-full overflow-auto"
            style={{ color: theme.tertiary }}
        >
            <TreeNode node={data.ast} theme={theme} />
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
        <div className="p-4 text-gray-300"><p>Contenido del explorador de archivos...</p></div>
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
    <div className="flex items-center border-b px-2" style={{ borderColor: theme.primary, backgroundColor: theme.secondary }}>
        {items.map((item, index) => (
            <button key={item} onClick={() => setActiveTab(index)}
                className={clsx("px-4 py-2 -mb-px text-sm font-medium border-b-2 transition-colors duration-200", { "border-blue-400 text-white": activeTab === index, "border-transparent text-gray-400 hover:text-white": activeTab !== index })}>
                {item}
            </button>
        ))}
    </div>
);

const DynamicAceStyles = ({ theme }) => {
    useEffect(() => {
        const styleId = 'dynamic-ace-theme-styles';
        // Limpia estilos antiguos para evitar conflictos
        document.getElementById(styleId)?.remove();

        if (!theme.syntax) return;

        const style = document.createElement('style');
        style.id = styleId;
        
        const s = theme.syntax;

        // Construye el CSS a partir del objeto de tema
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
    }, [theme]); // Se ejecuta cada vez que el tema cambia

    return null; // El componente no renderiza nada en el DOM
};


// --- Componente Principal de la App ---
function App() {
    const [action, setAction] = useState(null);
    const [themeCode, setThemeCode] = useState("pointerOfDoom");
    const [editorContent, setEditorContent] = useState("// ¡Bienvenido a Pointer of Doom!\n");
    const [filePath, setFilePath] = useState(null);
    const [tokens, setTokens] = useState({ tokens: [] });
    const [syntax, setSyntax] = useState({ ast: null, errors: [] });
    const [line, setLine] = useState(1);
    const [column, setColumn] = useState(1);
    const [analysisTab, setAnalysisTab] = useState(0);
    const [consoleTab, setConsoleTab] = useState(0);

    const aceEditorRef = useRef(null);
    const currentTheme = Themes[themeCode] || Themes.pointerOfDoom;

    useEffect(() => {
        if (aceEditorRef.current) {
            const editor = aceEditorRef.current.editor;
            const session = editor.getSession();
            const customMode = new DreamCMode();
            session.setMode(customMode);
            console.log("Modo personalizado 'DreamC' establecido.");
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
    }, [action]);

    useEffect(() => {
        const compileCode = async () => {
            if (editorContent.trim() === "") { setTokens({ tokens: [] }); setSyntax({ ast: null, errors: [] }); return; }
            try {
                const lexerResult = await window.electron.runLexer(editorContent);
                const parserResult = await window.electron.runParser(editorContent);
                setTokens(lexerResult);
                setSyntax(parserResult);
            } catch (error) { console.error("Error durante la compilación:", error); }
        };
        const debounce = setTimeout(compileCode, 500);
        return () => clearTimeout(debounce);
    }, [editorContent]);
    
    const handleAction = (newAction) => setAction(newAction);

    const renderAnalysisPanel = () => {
        switch (analysisTab) {
            case 0: return <TokenTable tokens={tokens} />;
            case 1: return <TreeView data={syntax} theme={Themes[themeCode]} />;
            case 2: return <div className="p-4 text-gray-400">Panel de Análisis Semántico (placeholder)</div>;
            case 3: return <div className="p-4 text-gray-400">Panel de Código Intermedio (placeholder)</div>;
            case 4: return <div className="p-4 text-gray-400">Panel de Tabla de Hash (placeholder)</div>;
            default: return null;
        }
    };
    
    const renderConsolePanel = () => {
        const ErrorsDisplay = ({ errors, type }) => (
            <div className="p-4 text-sm font-mono text-gray-300 overflow-auto h-full">
                {(!errors || errors.length === 0)
                    ? `No hay errores de ${type}.`
                    : errors.map((err, i) => <p key={i} className="text-red-400">Error: {err.message || JSON.stringify(err)} en línea {err.line} col {err.column}</p>)
                }
            </div>
        );
        switch (consoleTab) {
            case 0: return <ErrorsDisplay errors={tokens.tokens?.filter(t => t.token_type === "Invalid")} type="léxicos" />;
            case 1: return <ErrorsDisplay errors={syntax.errors} type="sintaxis" />;
            case 2: return <div className="p-4 text-gray-300">Salida de ejecución (placeholder)</div>;
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
                                <div className="flex flex-col h-full w-full bg-gray-900 rounded-tl-md overflow-hidden">
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
                                    <Tabs items={["Lexical", "Syntactic", "Semantic", "Intermediate Code", "Hash Table"]} activeTab={analysisTab} setActiveTab={setAnalysisTab} theme={currentTheme} />
                                    <div className="flex-grow overflow-auto" style={{ backgroundColor: currentTheme.secondarySemi }}>{renderAnalysisPanel()}</div>
                                </div>
                            </Panel>
                        </PanelGroup>
                    </Panel>
                    <PanelResizeHandle className="h-2 bg-gray-700 hover:bg-blue-600 transition-colors" />
                    <Panel defaultSize={25} minSize={10}>
                         <div className="flex flex-col h-full" style={{ backgroundColor: currentTheme.primary }}>
                            <Tabs items={["Lexical Errors", "Syntax Errors", "Execution"]} activeTab={consoleTab} setActiveTab={setConsoleTab} theme={currentTheme} />
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
