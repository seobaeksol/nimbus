import React, { useState, useEffect, useRef } from 'react';
import './ConnectionManager.css';

// Connection types based on remote-fs crate
export interface RemoteConnection {
  id: string;
  name: string;
  protocol: 'sftp' | 'ftp' | 'ftps' | 'webdav' | 'webdavs';
  host: string;
  port?: number;
  username: string;
  password?: string;
  privateKeyPath?: string;
  privateKeyPassphrase?: string;
  timeout?: number;
  usePassiveFtp?: boolean;
  verifySsl?: boolean;
  basePath?: string;
  createdAt?: Date;
  lastUsed?: Date;
  isConnected?: boolean;
}

export interface ConnectionFormData {
  name: string;
  protocol: 'sftp' | 'ftp' | 'ftps' | 'webdav' | 'webdavs';
  host: string;
  port: string;
  username: string;
  password: string;
  privateKeyPath: string;
  privateKeyPassphrase: string;
  timeout: string;
  usePassiveFtp: boolean;
  verifySsl: boolean;
  basePath: string;
}

interface ConnectionManagerProps {
  isOpen: boolean;
  connections: RemoteConnection[];
  onClose: () => void;
  onSave: (connection: RemoteConnection) => Promise<void>;
  onDelete: (connectionId: string) => Promise<void>;
  onTest: (connection: RemoteConnection) => Promise<boolean>;
  onConnect: (connectionId: string) => Promise<void>;
  onDisconnect: (connectionId: string) => Promise<void>;
}

const ConnectionManager: React.FC<ConnectionManagerProps> = ({
  isOpen,
  connections,
  onClose,
  onSave,
  onDelete,
  onTest,
  onConnect,
  onDisconnect,
}) => {
  const [selectedConnection, setSelectedConnection] = useState<RemoteConnection | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [formData, setFormData] = useState<ConnectionFormData>({
    name: '',
    protocol: 'sftp',
    host: '',
    port: '',
    username: '',
    password: '',
    privateKeyPath: '',
    privateKeyPassphrase: '',
    timeout: '30',
    usePassiveFtp: true,
    verifySsl: true,
    basePath: '',
  });
  const [testResults, setTestResults] = useState<Record<string, { success: boolean; message: string }>>({});
  const [loading, setLoading] = useState<Record<string, boolean>>({});
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Reset form when dialog closes
  useEffect(() => {
    if (!isOpen) {
      setSelectedConnection(null);
      setIsEditing(false);
      setIsCreating(false);
      resetForm();
    }
  }, [isOpen]);

  const resetForm = () => {
    setFormData({
      name: '',
      protocol: 'sftp',
      host: '',
      port: '',
      username: '',
      password: '',
      privateKeyPath: '',
      privateKeyPassphrase: '',
      timeout: '30',
      usePassiveFtp: true,
      verifySsl: true,
      basePath: '',
    });
  };

  const getDefaultPort = (protocol: string): string => {
    switch (protocol) {
      case 'sftp': return '22';
      case 'ftp': return '21';
      case 'ftps': return '990';
      case 'webdav': return '80';
      case 'webdavs': return '443';
      default: return '';
    }
  };

  const handleProtocolChange = (protocol: ConnectionFormData['protocol']) => {
    setFormData(prev => ({
      ...prev,
      protocol,
      port: getDefaultPort(protocol),
    }));
  };

  const handleInputChange = (field: keyof ConnectionFormData, value: string | boolean) => {
    setFormData(prev => ({
      ...prev,
      [field]: value,
    }));
  };

  const handleFileSelect = () => {
    fileInputRef.current?.click();
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setFormData(prev => ({ ...prev, privateKeyPath: file.path || file.name }));
    }
  };

  const handleSave = async () => {
    if (!formData.name || !formData.host || !formData.username) {
      return;
    }

    const connection: RemoteConnection = {
      id: selectedConnection?.id || Date.now().toString(),
      name: formData.name,
      protocol: formData.protocol,
      host: formData.host,
      port: formData.port ? parseInt(formData.port) : undefined,
      username: formData.username,
      password: formData.password || undefined,
      privateKeyPath: formData.privateKeyPath || undefined,
      privateKeyPassphrase: formData.privateKeyPassphrase || undefined,
      timeout: formData.timeout ? parseInt(formData.timeout) : undefined,
      usePassiveFtp: formData.usePassiveFtp,
      verifySsl: formData.verifySsl,
      basePath: formData.basePath || undefined,
      createdAt: selectedConnection?.createdAt || new Date(),
    };

    try {
      setLoading(prev => ({ ...prev, save: true }));
      await onSave(connection);
      setIsEditing(false);
      setIsCreating(false);
      resetForm();
    } finally {
      setLoading(prev => ({ ...prev, save: false }));
    }
  };

  const handleEdit = (connection: RemoteConnection) => {
    setSelectedConnection(connection);
    setFormData({
      name: connection.name,
      protocol: connection.protocol,
      host: connection.host,
      port: connection.port?.toString() || '',
      username: connection.username,
      password: connection.password || '',
      privateKeyPath: connection.privateKeyPath || '',
      privateKeyPassphrase: connection.privateKeyPassphrase || '',
      timeout: connection.timeout?.toString() || '30',
      usePassiveFtp: connection.usePassiveFtp ?? true,
      verifySsl: connection.verifySsl ?? true,
      basePath: connection.basePath || '',
    });
    setIsEditing(true);
  };

  const handleTest = async (connection: RemoteConnection) => {
    try {
      setLoading(prev => ({ ...prev, [`test_${connection.id}`]: true }));
      const success = await onTest(connection);
      setTestResults(prev => ({
        ...prev,
        [connection.id]: {
          success,
          message: success ? 'Connection successful' : 'Connection failed',
        },
      }));
    } catch (error: any) {
      setTestResults(prev => ({
        ...prev,
        [connection.id]: {
          success: false,
          message: error.message || 'Connection failed',
        },
      }));
    } finally {
      setLoading(prev => ({ ...prev, [`test_${connection.id}`]: false }));
    }
  };

  const handleConnect = async (connectionId: string) => {
    try {
      setLoading(prev => ({ ...prev, [`connect_${connectionId}`]: true }));
      await onConnect(connectionId);
    } finally {
      setLoading(prev => ({ ...prev, [`connect_${connectionId}`]: false }));
    }
  };

  const handleDisconnect = async (connectionId: string) => {
    try {
      setLoading(prev => ({ ...prev, [`disconnect_${connectionId}`]: true }));
      await onDisconnect(connectionId);
    } finally {
      setLoading(prev => ({ ...prev, [`disconnect_${connectionId}`]: false }));
    }
  };

  const handleDelete = async (connectionId: string) => {
    if (window.confirm('Are you sure you want to delete this connection?')) {
      try {
        setLoading(prev => ({ ...prev, [`delete_${connectionId}`]: true }));
        await onDelete(connectionId);
        if (selectedConnection?.id === connectionId) {
          setSelectedConnection(null);
          setIsEditing(false);
        }
      } finally {
        setLoading(prev => ({ ...prev, [`delete_${connectionId}`]: false }));
      }
    }
  };

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div 
      className="connection-manager-backdrop" 
      onClick={handleBackdropClick}
      onKeyDown={handleKeyDown}
      tabIndex={-1}
    >
      <div className="connection-manager">
        <div className="connection-manager-header">
          <h2 className="connection-manager-title">Remote Connections</h2>
          <button 
            className="connection-manager-close-button"
            onClick={onClose}
            type="button"
          >
            ✕
          </button>
        </div>
        
        <div className="connection-manager-body">
          {/* Connection List */}
          <div className="connection-list-panel">
            <div className="connection-list-header">
              <h3>Connections</h3>
              <button 
                className="connection-add-button"
                onClick={() => {
                  resetForm();
                  setIsCreating(true);
                  setIsEditing(false);
                }}
                type="button"
              >
                Add New
              </button>
            </div>
            
            <div className="connection-list">
              {connections.map(connection => (
                <div 
                  key={connection.id} 
                  className={`connection-item ${selectedConnection?.id === connection.id ? 'selected' : ''}`}
                  onClick={() => setSelectedConnection(connection)}
                >
                  <div className="connection-item-header">
                    <div className="connection-name">{connection.name}</div>
                    <div className={`connection-status ${connection.isConnected ? 'connected' : 'disconnected'}`}>
                      {connection.isConnected ? '●' : '○'}
                    </div>
                  </div>
                  
                  <div className="connection-details">
                    <span className="protocol-badge">{connection.protocol.toUpperCase()}</span>
                    <span className="connection-host">{connection.username}@{connection.host}</span>
                  </div>
                  
                  {testResults[connection.id] && (
                    <div className={`test-result ${testResults[connection.id].success ? 'success' : 'error'}`}>
                      {testResults[connection.id].message}
                    </div>
                  )}
                  
                  <div className="connection-actions">
                    <button 
                      className="action-button test"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleTest(connection);
                      }}
                      disabled={loading[`test_${connection.id}`]}
                      type="button"
                    >
                      {loading[`test_${connection.id}`] ? 'Testing...' : 'Test'}
                    </button>
                    
                    {connection.isConnected ? (
                      <button 
                        className="action-button disconnect"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDisconnect(connection.id);
                        }}
                        disabled={loading[`disconnect_${connection.id}`]}
                        type="button"
                      >
                        {loading[`disconnect_${connection.id}`] ? 'Disconnecting...' : 'Disconnect'}
                      </button>
                    ) : (
                      <button 
                        className="action-button connect"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleConnect(connection.id);
                        }}
                        disabled={loading[`connect_${connection.id}`]}
                        type="button"
                      >
                        {loading[`connect_${connection.id}`] ? 'Connecting...' : 'Connect'}
                      </button>
                    )}
                    
                    <button 
                      className="action-button edit"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleEdit(connection);
                      }}
                      type="button"
                    >
                      Edit
                    </button>
                    
                    <button 
                      className="action-button delete"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(connection.id);
                      }}
                      disabled={loading[`delete_${connection.id}`]}
                      type="button"
                    >
                      {loading[`delete_${connection.id}`] ? 'Deleting...' : 'Delete'}
                    </button>
                  </div>
                </div>
              ))}
              
              {connections.length === 0 && (
                <div className="empty-state">
                  <p>No connections configured.</p>
                  <p>Click "Add New" to create your first remote connection.</p>
                </div>
              )}
            </div>
          </div>
          
          {/* Connection Form */}
          {(isEditing || isCreating) && (
            <div className="connection-form-panel">
              <div className="connection-form-header">
                <h3>{isCreating ? 'Add New Connection' : 'Edit Connection'}</h3>
                <button 
                  className="form-close-button"
                  onClick={() => {
                    setIsEditing(false);
                    setIsCreating(false);
                    resetForm();
                  }}
                  type="button"
                >
                  ✕
                </button>
              </div>
              
              <form className="connection-form" onSubmit={(e) => e.preventDefault()}>
                <div className="form-group">
                  <label htmlFor="connection-name">Connection Name *</label>
                  <input
                    id="connection-name"
                    type="text"
                    value={formData.name}
                    onChange={(e) => handleInputChange('name', e.target.value)}
                    placeholder="My Server"
                    required
                  />
                </div>
                
                <div className="form-group">
                  <label htmlFor="protocol">Protocol *</label>
                  <select
                    id="protocol"
                    value={formData.protocol}
                    onChange={(e) => handleProtocolChange(e.target.value as ConnectionFormData['protocol'])}
                  >
                    <option value="sftp">SFTP (SSH File Transfer)</option>
                    <option value="ftp">FTP (File Transfer Protocol)</option>
                    <option value="ftps">FTPS (FTP over SSL/TLS)</option>
                    <option value="webdav">WebDAV (HTTP)</option>
                    <option value="webdavs">WebDAV (HTTPS)</option>
                  </select>
                </div>
                
                <div className="form-row">
                  <div className="form-group">
                    <label htmlFor="host">Host *</label>
                    <input
                      id="host"
                      type="text"
                      value={formData.host}
                      onChange={(e) => handleInputChange('host', e.target.value)}
                      placeholder="example.com"
                      required
                    />
                  </div>
                  
                  <div className="form-group">
                    <label htmlFor="port">Port</label>
                    <input
                      id="port"
                      type="number"
                      value={formData.port}
                      onChange={(e) => handleInputChange('port', e.target.value)}
                      placeholder={getDefaultPort(formData.protocol)}
                    />
                  </div>
                </div>
                
                <div className="form-group">
                  <label htmlFor="username">Username *</label>
                  <input
                    id="username"
                    type="text"
                    value={formData.username}
                    onChange={(e) => handleInputChange('username', e.target.value)}
                    placeholder="username"
                    required
                  />
                </div>
                
                {/* Authentication section */}
                <div className="form-section">
                  <h4>Authentication</h4>
                  
                  {formData.protocol === 'sftp' && (
                    <>
                      <div className="form-group">
                        <label htmlFor="password">Password</label>
                        <input
                          id="password"
                          type="password"
                          value={formData.password}
                          onChange={(e) => handleInputChange('password', e.target.value)}
                          placeholder="Leave empty to use key authentication"
                        />
                      </div>
                      
                      <div className="form-group">
                        <label htmlFor="private-key">Private Key File</label>
                        <div className="file-input-group">
                          <input
                            id="private-key"
                            type="text"
                            value={formData.privateKeyPath}
                            onChange={(e) => handleInputChange('privateKeyPath', e.target.value)}
                            placeholder="Path to private key file"
                          />
                          <button
                            type="button"
                            className="file-select-button"
                            onClick={handleFileSelect}
                          >
                            Browse
                          </button>
                          <input
                            ref={fileInputRef}
                            type="file"
                            accept=".pem,.key,.ppk"
                            onChange={handleFileChange}
                            style={{ display: 'none' }}
                          />
                        </div>
                      </div>
                      
                      {formData.privateKeyPath && (
                        <div className="form-group">
                          <label htmlFor="key-passphrase">Key Passphrase</label>
                          <input
                            id="key-passphrase"
                            type="password"
                            value={formData.privateKeyPassphrase}
                            onChange={(e) => handleInputChange('privateKeyPassphrase', e.target.value)}
                            placeholder="Leave empty if key is not encrypted"
                          />
                        </div>
                      )}
                    </>
                  )}
                  
                  {(formData.protocol === 'ftp' || formData.protocol === 'ftps' || formData.protocol === 'webdav' || formData.protocol === 'webdavs') && (
                    <div className="form-group">
                      <label htmlFor="password">Password *</label>
                      <input
                        id="password"
                        type="password"
                        value={formData.password}
                        onChange={(e) => handleInputChange('password', e.target.value)}
                        placeholder="Password"
                        required
                      />
                    </div>
                  )}
                </div>
                
                {/* Advanced options */}
                <div className="form-section">
                  <h4>Advanced Options</h4>
                  
                  <div className="form-group">
                    <label htmlFor="timeout">Connection Timeout (seconds)</label>
                    <input
                      id="timeout"
                      type="number"
                      value={formData.timeout}
                      onChange={(e) => handleInputChange('timeout', e.target.value)}
                      placeholder="30"
                      min="1"
                      max="300"
                    />
                  </div>
                  
                  {formData.protocol === 'ftp' && (
                    <div className="form-group checkbox">
                      <label>
                        <input
                          type="checkbox"
                          checked={formData.usePassiveFtp}
                          onChange={(e) => handleInputChange('usePassiveFtp', e.target.checked)}
                        />
                        Use Passive FTP Mode
                      </label>
                    </div>
                  )}
                  
                  {(formData.protocol === 'ftps' || formData.protocol === 'webdavs') && (
                    <div className="form-group checkbox">
                      <label>
                        <input
                          type="checkbox"
                          checked={formData.verifySsl}
                          onChange={(e) => handleInputChange('verifySsl', e.target.checked)}
                        />
                        Verify SSL Certificate
                      </label>
                    </div>
                  )}
                  
                  <div className="form-group">
                    <label htmlFor="base-path">Initial Directory</label>
                    <input
                      id="base-path"
                      type="text"
                      value={formData.basePath}
                      onChange={(e) => handleInputChange('basePath', e.target.value)}
                      placeholder="/path/to/initial/directory"
                    />
                  </div>
                </div>
                
                <div className="form-actions">
                  <button 
                    className="form-button cancel"
                    onClick={() => {
                      setIsEditing(false);
                      setIsCreating(false);
                      resetForm();
                    }}
                    type="button"
                  >
                    Cancel
                  </button>
                  
                  <button 
                    className="form-button save"
                    onClick={handleSave}
                    disabled={loading.save || !formData.name || !formData.host || !formData.username}
                    type="submit"
                  >
                    {loading.save ? 'Saving...' : (isCreating ? 'Create Connection' : 'Save Changes')}
                  </button>
                </div>
              </form>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default ConnectionManager;