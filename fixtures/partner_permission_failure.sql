-- Inject a storage failure after the membership write to exercise transaction rollback.
CREATE FUNCTION fail_test_partner_permission() RETURNS trigger AS $$
BEGIN
    IF NEW.user_id = 103 AND NEW.permission = 'zones.view' THEN
        RAISE EXCEPTION 'injected permission storage failure';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER test_partner_permission_failure
BEFORE INSERT ON partner_member_permissions
FOR EACH ROW EXECUTE FUNCTION fail_test_partner_permission();
