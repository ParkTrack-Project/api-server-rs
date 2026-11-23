UPDATE partner_memberships
SET read_scope = CASE
    WHEN read_scope IN ('own', 'owned', 'own_or_assigned') THEN 'own'
    WHEN read_scope IN ('partner_all', 'all') THEN 'partner_all'
    ELSE 'none'
END
WHERE read_scope IS NULL OR read_scope NOT IN ('own', 'partner_all', 'none');

UPDATE partner_memberships
SET write_scope = CASE
    WHEN write_scope IN ('own', 'owned', 'own_or_assigned') THEN 'own'
    WHEN write_scope IN ('partner_all', 'all') THEN 'partner_all'
    ELSE 'none'
END
WHERE write_scope IS NULL OR write_scope NOT IN ('own', 'partner_all', 'none');

UPDATE partner_memberships
SET delete_scope = CASE
    WHEN delete_scope IN ('own', 'owned', 'own_or_assigned') THEN 'own'
    WHEN delete_scope IN ('partner_all', 'all') THEN 'partner_all'
    ELSE 'none'
END
WHERE delete_scope IS NULL OR delete_scope NOT IN ('own', 'partner_all', 'none');

ALTER TABLE partner_memberships
    ALTER COLUMN read_scope SET NOT NULL,
    ALTER COLUMN write_scope SET NOT NULL,
    ALTER COLUMN delete_scope SET NOT NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'partner_memberships'::regclass AND conname = 'chk_partner_memberships_read_scope'
    ) THEN
        ALTER TABLE partner_memberships
            ADD CONSTRAINT chk_partner_memberships_read_scope CHECK (read_scope IN ('own', 'partner_all', 'none'));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'partner_memberships'::regclass AND conname = 'chk_partner_memberships_write_scope'
    ) THEN
        ALTER TABLE partner_memberships
            ADD CONSTRAINT chk_partner_memberships_write_scope CHECK (write_scope IN ('own', 'partner_all', 'none'));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'partner_memberships'::regclass AND conname = 'chk_partner_memberships_delete_scope'
    ) THEN
        ALTER TABLE partner_memberships
            ADD CONSTRAINT chk_partner_memberships_delete_scope CHECK (delete_scope IN ('own', 'partner_all', 'none'));
    END IF;
END $$;
