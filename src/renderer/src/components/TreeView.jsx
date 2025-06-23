import React, { useState } from 'react';

const TreeNode = ({ node, depth = 0 }) => {
  const [isExpanded, setIsExpanded] = useState(true);
  const hasChildren = node.children && node.children.length > 0;
  
  return (
    <div className="tree-node" style={{ marginLeft: `${depth * 15}px` }}>
      <div 
        className="node-card"
        onClick={() => hasChildren && setIsExpanded(!isExpanded)}
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

// Add this CSS to your stylesheet
const styles = `
/* Tree View Styling */
.tree-view-container {
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 14px;
  padding: 12px;
  background: #f8fafc;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
  overflow: auto;
  max-height: 70vh; /* Limit height */
  max-width: 100%; /* Prevent horizontal overflow */
}

.tree-node {
  margin-bottom: 4px;
  min-width: max-content; /* Prevent squeezing */
}

.node-card {
  padding: 8px 12px;
  background: white;
  border-radius: 6px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 1px 2px rgba(0,0,0,0.03);
  transition: all 0.2s ease;
  width: fit-content; /* Shrink to content width */
  min-width: 200px; /* Minimum width for cards */
}

.node-card:hover {
  background: #f1f5f9;
  border-color: #cbd5e1;
  box-shadow: 0 2px 4px rgba(0,0,0,0.05);
  transform: translateY(-1px);
}

.node-header {
  display: flex;
  align-items: center;
  cursor: ${props => props.hasChildren ? 'pointer' : 'default'};
}

.expand-icon {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #edf2f7;
  border-radius: 4px;
  margin-right: 8px;
  font-size: 12px;
  color: #4a5568;
  flex-shrink: 0; /* Prevent shrinking */
}

.node-content {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
}

.node-type {
  font-weight: 600;
  color: #2d3748;
  white-space: nowrap; /* Keep node type on one line */
}

.node-value {
  color: #2b6cb0;
  font-weight: 500;
  word-break: break-word; /* Allow breaking long values */
}

.node-location {
  color: #718096;
  font-size: 12px;
  background: #edf2f7;
  padding: 1px 6px;
  border-radius: 4px;
  margin-left: 4px;
  white-space: nowrap;
}

.tree-empty-message {
  padding: 20px;
  text-align: center;
  color: #718096;
}

.tree-errors {
  margin-top: 15px;
  padding: 12px;
  background: #fff5f5;
  border-radius: 6px;
  border: 1px solid #fed7d7;
}

.tree-errors h4 {
  margin: 0 0 8px 0;
  color: #e53e3e;
}

.tree-errors ul {
  margin: 0;
  padding-left: 20px;
}

.tree-errors li {
  color: #c53030;
  font-size: 13px;
}

/* Ensure proper spacing for children */
.node-children {
  margin-top: 8px;
  padding-left: 10px;
  border-left: 1px dashed #e2e8f0;
}
`;

// Inject styles (do this once in your app)
if (!document.getElementById('tree-view-styles')) {
  const styleSheet = document.createElement('style');
  styleSheet.id = 'tree-view-styles';
  styleSheet.innerHTML = styles;
  document.head.appendChild(styleSheet);
}