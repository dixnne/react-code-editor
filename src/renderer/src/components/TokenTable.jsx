import { Box } from '@chakra-ui/react';
import React from 'react';

function TokenTable({
    tokens,
    tableClassName = 'table table-striped table-bordered table-hover table-sm mb-0',
    // Añadimos position: 'relative' al wrapper para que el sticky header funcione correctamente dentro de él.
    // Añadimos un pequeño padding inferior para evitar que la última fila se corte.
    wrapperStyle = { maxHeight: '345px', overflowY: 'auto', position: 'relative', paddingBottom: '1px' },
    // Estilo para el encabezado, asegurando un fondo para cubrir el contenido al hacer scroll.
    headerStyle = { position: 'sticky', top: 0, zIndex: 1, backgroundColor: '#343a40' }, // Usando el color oscuro de thead-dark
    emptyMessage = 'No hay tokens para mostrar.'
  }) {
  
    if (!tokens.tokens || tokens.tokens.length === 0) {
      return (
        <div className="alert alert-info mt-2" role="alert"> {/* Añadido margen superior */}
          {emptyMessage}
        </div>
      );
    }
  
    return (
      // Contenedor externo para responsiveness horizontal (opcional)
      // Si no necesitas scroll horizontal, puedes quitar 'table-responsive'
      <div className="table-responsive">
        {/* Contenedor interno para scroll vertical */}
        <div style={wrapperStyle}>
          <table className={tableClassName}>
            {/* Encabezado Pegajoso (Sticky Header) */}
            {/* Aplicamos el estilo directamente y usamos la clase thead-dark para colores */}
            <thead className="bg-light" style={headerStyle}>
              <tr>
                {/* Aseguramos que las celdas del encabezado también tengan fondo */}
                <th scope="col" style={{ backgroundColor: 'inherit' }}>Tipo de Token</th>
                <th scope="col" style={{ backgroundColor: 'inherit' }}>Lexema</th>
                <th scope="col" style={{ backgroundColor: 'inherit' }}>Línea</th>
                <th scope="col" style={{ backgroundColor: 'inherit' }}>Columna</th>
              </tr>
            </thead>
            <tbody>
              {tokens.tokens.map((token, index) => {
              if(token.token_type !== "Invalid" && token.tokenType !== "CommentSingle" && token.tokenType !== "CommentMultiLine" && token.tokenType !== "NewLine") {
                return (
                  <tr key={index}>
                    <td>{token.token_type}</td>
                    <td>{token.lexeme}</td>
                    <td>{token.line}</td>
                    <td>{token.column}</td>
                  </tr>
                )
              } else {
                return ("")
              }
            })}
            </tbody>
          </table>
        </div>
      </div>
    );
  }

export default TokenTable;