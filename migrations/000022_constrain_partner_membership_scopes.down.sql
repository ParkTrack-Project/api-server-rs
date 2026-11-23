ALTER TABLE partner_memberships
    DROP CONSTRAINT IF EXISTS chk_partner_memberships_read_scope,
    DROP CONSTRAINT IF EXISTS chk_partner_memberships_write_scope,
    DROP CONSTRAINT IF EXISTS chk_partner_memberships_delete_scope,
    ALTER COLUMN read_scope DROP NOT NULL,
    ALTER COLUMN write_scope DROP NOT NULL,
    ALTER COLUMN delete_scope DROP NOT NULL;
