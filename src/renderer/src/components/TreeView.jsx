import React, { useState } from 'react';

const TreeNode = ({ node, depth = 0, wrapperStyle = { maxHeight: '332px', overflowY: 'auto', position: 'relative', paddingBottom: '1px' } }) => {
  const [isExpanded, setIsExpanded] = useState(true);
  const hasChildren = node.children && node.children.length > 0;
  
  return (
    <div className="tree-node" style={ wrapperStyle }>
      <div 
        className="node-card"
        onClick={() => hasChildren && setIsExpanded(!isExpanded)}
        role='button'
      >
        <div className="node-header">
          {hasChildren && (
            <div className="expand-icon">
              {isExpanded ? '▼' : '►'}
            </div>
          )}
          <div className="node-content">
            <span className="node-type">{node.node_type}</span>
            {node.value && <span className="node-value">: {node.value}</span>}
            
            {(node.start_line !== undefined && node.start_line > 0) && (
              <span className="node-location">
                (L{node.start_line}:{node.start_column})
              </span>
            )}
          </div>
        </div>
      </div>
      
      {isExpanded && hasChildren && (
        <div className="node-children">
          {node.children.map((child, index) => (
            <TreeNode 
              key={index} 
              node={child} 
              depth={depth + 1} 
            />
          ))}
        </div>
      )}
    </div>
  );
};

const TreeView = ({ data }) => {
  if (!data || !data.ast) {
    return (
      <div className="tree-view-container">
        <div className="tree-empty-message">
          {!data ? 'No data provided' : 'AST not available'}
          {data?.errors?.length > 0 && (
            <div className="tree-errors">
              <h4>Errors:</h4>
              <ul>
                {data.errors.map((error, i) => (
                  <li key={i}>{error}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="tree-view-container">
      <TreeNode node={data.ast} />
    </div>
  );
};

export default TreeView;
