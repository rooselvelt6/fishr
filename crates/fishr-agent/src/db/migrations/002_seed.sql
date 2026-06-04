-- 002_seed.sql
-- Seed data for a fresh install

-- Default fish types (catálogo central)
-- Uses deterministic ULID-like IDs for reproducibility across installs
INSERT OR IGNORE INTO fish_type (id, name, species, category, description, op_counter, updated_at)
VALUES
    ('01AR000001', 'Merluza', 'Merluccius merluccius', 'White', 'Pescado blanco de carne suave', 1, datetime('now')),
    ('01AR000002', 'Lenguado', 'Solea solea', 'White', 'Pescado plano de carne fina', 1, datetime('now')),
    ('01AR000003', 'Atún', 'Thunnus thynnus', 'Blue', 'Pescado azul de carne roja', 1, datetime('now')),
    ('01AR000004', 'Pargo', 'Lutjanus campechanus', 'White', 'Pescado blanco de roca', 1, datetime('now')),
    ('01AR000005', 'Dorado', 'Coryphaena hippurus', 'White', 'Pescado de carne firme', 1, datetime('now')),
    ('01AR000006', 'Sardina', 'Sardina pilchardus', 'Blue', 'Pescado azul pequeño', 1, datetime('now')),
    ('01AR000007', 'Camarón', 'Penaeus vannamei', 'Crustacean', 'Crustáceo de carne dulce', 1, datetime('now')),
    ('01AR000008', 'Pulpeta', 'Octopus vulgaris', 'Shellfish', 'Molusco de carne firme', 1, datetime('now')),
    ('01AR000009', 'Cazón', 'Galeorhinus galeus', 'White', 'Tiburón pequeño de carne blanca', 1, datetime('now')),
    ('01AR000010', 'Curvina', 'Cynoscion virescens', 'White', 'Pescado blanco de agua salada', 1, datetime('now'));
