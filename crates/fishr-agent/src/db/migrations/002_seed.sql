-- 002_seed.sql
-- Seed data for a fresh install

-- Default fish types (catálogo central)
INSERT OR IGNORE INTO fish_type (id, name, species, category, description, op_counter, updated_at)
VALUES
    ('ft_default_001', 'Merluza', 'Merluccius merluccius', 'White', 'Pescado blanco de carne suave', 1, datetime('now')),
    ('ft_default_002', 'Lenguado', 'Solea solea', 'White', 'Pescado plano de carne fina', 1, datetime('now')),
    ('ft_default_003', 'Atún', 'Thunnus thynnus', 'Blue', 'Pescado azul de carne roja', 1, datetime('now')),
    ('ft_default_004', 'Pargo', 'Lutjanus campechanus', 'White', 'Pescado blanco de roca', 1, datetime('now')),
    ('ft_default_005', 'Dorado', 'Coryphaena hippurus', 'White', 'Pescado de carne firme', 1, datetime('now')),
    ('ft_default_006', 'Sardina', 'Sardina pilchardus', 'Blue', 'Pescado azul pequeño', 1, datetime('now')),
    ('ft_default_007', 'Camarón', 'Penaeus vannamei', 'Crustacean', 'Crustáceo de carne dulce', 1, datetime('now')),
    ('ft_default_008', 'Pulpeta', 'Octopus vulgaris', 'Shellfish', 'Molusco de carne firme', 1, datetime('now')),
    ('ft_default_009', 'Cazón', 'Galeorhinus galeus', 'White', 'Tiburón pequeño de carne blanca', 1, datetime('now')),
    ('ft_default_010', 'Curvina', 'Cynoscion virescens', 'White', 'Pescado blanco de agua salada', 1, datetime('now'));
