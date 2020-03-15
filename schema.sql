CREATE TABLE InterfaceMetadata (
    Id INTEGER PRIMARY KEY AUTOINCREMENT,
    InterfaceName TEXT NOT NULL,
    ReceivedBytes INTEGER,
    TransmittedBytes INTEGER,
    Timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);