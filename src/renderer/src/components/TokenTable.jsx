// src/components/TokenTable.jsx
import React from 'react';

function TokenTable({ tokens, emptyMessage = 'No hay tokens para mostrar.' }) {
  const validTokens = tokens.tokens?.filter(token =>
    token.token_type !== "Invalid" &&
    token.tokenType !== "CommentSingle" &&
    token.tokenType !== "CommentMultiLine" &&
    token.tokenType !== "NewLine"
  ) || [];

  if (validTokens.length === 0) {
    return (
      <div className="p-4 text-center text-gray-400">
        {emptyMessage}
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto">
      <table className="w-full text-sm text-left text-gray-300">
        <thead className="text-xs text-gray-300 uppercase bg-gray-700 sticky top-0">
          <tr>
            <th scope="col" className="px-6 py-3">Tipo</th>
            <th scope="col" className="px-6 py-3">Lexema</th>
            <th scope="col" className="px-6 py-3">Línea</th>
            <th scope="col" className="px-6 py-3">Columna</th>
          </tr>
        </thead>
        <tbody>
          {validTokens.map((token, index) => (
            <tr key={index} className="border-b bg-gray-800 border-gray-700 hover:bg-gray-600">
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
}

export default TokenTable;