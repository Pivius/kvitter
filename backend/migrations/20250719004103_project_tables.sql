CREATE TYPE project_status AS ENUM (
    'ACTIVE', 'COMPLETED', 'ON_HOLD', 'ARCHIVED'
);


CREATE TABLE clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    first_name TEXT,
	last_name TEXT,
    phone TEXT,
    company_name TEXT,
    address_line1 TEXT,
    address_line2 TEXT,
    city TEXT,
    postal_code TEXT,
    country TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE projects (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	name TEXT NOT NULL,
	description TEXT,
	client_id UUID NOT NULL REFERENCES clients(id),
	total_budget DECIMAL,
	default_hourly_rate DECIMAL,
	is_fixed_price BOOLEAN NOT NULL,
	start_date DATE,
	end_date DATE,
	status project_status NOT NULL DEFAULT 'ACTIVE',
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	created_by UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE project_members (
	project_id UUID REFERENCES projects(id),
	user_id UUID REFERENCES users(id),
	role TEXT NOT NULL,
	hourly_rate DECIMAL,
	joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	PRIMARY KEY (project_id, user_id)
);

CREATE TABLE jobs (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	project_id UUID NOT NULL REFERENCES projects(id),
	name TEXT NOT NULL,
	description TEXT,
	budget DECIMAL,
	is_fixed_price BOOLEAN NOT NULL,
	status project_status NOT NULL DEFAULT 'ACTIVE',
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE time_entries (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	job_id UUID NOT NULL REFERENCES jobs(id),
	user_id UUID NOT NULL REFERENCES users(id),
	time_spent INTERVAL NOT NULL,
	description TEXT,
	entry_date DATE NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE milestones (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	project_id UUID NOT NULL REFERENCES projects(id),
	description TEXT NOT NULL,
	amount DECIMAL NOT NULL,
	due_date DATE,
	completed_at TIMESTAMPTZ,
	status project_status NOT NULL DEFAULT 'ACTIVE',
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);