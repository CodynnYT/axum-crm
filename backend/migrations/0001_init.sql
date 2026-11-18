-- Axum CRM schema

-- People who can sign in
CREATE TABLE users (
-- A number Postgres counts up by itself: 1, 2, 3...
       id            BIGSERIAL PRIMARY KEY,
-- The display name; NOT NULL means it must always be filled in
       name          TEXT        NOT NULL,
-- The email address, and no two users may share one
       email         TEXT        NOT NULL UNIQUE,
-- The scrambled password (never the real one)
       password_hash TEXT        NOT NULL,
-- Either 'admin' or 'member'; CHECK makes the database refuse anything else
       role          TEXT        NOT NULL DEFAULT 'member' CHECK (role IN ('admin', 'member')),
-- When the row was made; NOW() fills it in automatically
       created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- People who might buy
CREATE TABLE leads (
-- The lead's ID number
       id         BIGSERIAL PRIMARY KEY,
-- The lead's name (required)
       name       TEXT        NOT NULL,
-- Optional fields default to empty text instead of "nothing"
       company    TEXT        NOT NULL DEFAULT '',
-- The lead's email
       email      TEXT        NOT NULL DEFAULT '',
-- The lead's phone number
       phone      TEXT        NOT NULL DEFAULT '',
-- Where the lead came from: only these six values are allowed
       source     TEXT        NOT NULL DEFAULT 'Website'
           CHECK (source IN ('Website', 'Referral', 'Social Media', 'Email', 'Cold Call', 'Other')),
-- How far along they are
       stage      TEXT        NOT NULL DEFAULT 'New'
           CHECK (stage IN ('New', 'Contacted', 'Qualified', 'Unqualified')),
-- Whether we're still working on them
       status     TEXT        NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive')),
-- When the lead was added
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- People who buy
CREATE TABLE customers (
-- The customer's ID number
           id         BIGSERIAL PRIMARY KEY,
-- The customer's name (required)
           name       TEXT        NOT NULL,
-- Their company
           company    TEXT        NOT NULL DEFAULT '',
-- Their email
           email      TEXT        NOT NULL DEFAULT '',
-- Their phone number
           phone      TEXT        NOT NULL DEFAULT '',
-- Free-form notes
           notes      TEXT        NOT NULL DEFAULT '',
-- When the customer was added
           created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Money on the way
CREATE TABLE deals (
-- The deal's ID number
       id             BIGSERIAL PRIMARY KEY,
-- The deal's name (required)
       name           TEXT             NOT NULL,
-- Which customer it belongs to; if that customer is deleted, this becomes empty
       customer_id    BIGINT           REFERENCES customers (id) ON DELETE SET NULL,
-- The price; it can never be negative
       amount         DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (amount >= 0),
-- Where the deal is in the pipeline
       stage          TEXT             NOT NULL DEFAULT 'New'
           CHECK (stage IN ('New', 'Qualified', 'Proposal', 'Negotiation', 'Won', 'Lost')),
-- When we expect to close it (a date, optional)
       expected_close DATE,
-- When the deal was added
       created_at     TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

-- Calls, meetings, emails and tasks
CREATE TABLE activities (
-- The activity's ID number
            id            BIGSERIAL PRIMARY KEY,
-- What it is, like "Call with Sarah"
            title         TEXT        NOT NULL,
-- Who it's about, as plain text
            related_to    TEXT        NOT NULL DEFAULT '',
-- Only these four kinds are allowed
            activity_type TEXT        NOT NULL DEFAULT 'Call'
                CHECK (activity_type IN ('Call', 'Meeting', 'Email', 'Task')),
-- When it's due (optional)
            due_at        TIMESTAMPTZ,
-- Done or not
            status        TEXT        NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'Completed')),
-- What happened
            notes         TEXT        NOT NULL DEFAULT '',
-- Which user created it; if they're deleted, this becomes empty
            created_by    BIGINT      REFERENCES users (id) ON DELETE SET NULL,
-- When it was added
            created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Saved date ranges for the sales summary
CREATE TABLE reports (
-- The report's ID number
         id         BIGSERIAL PRIMARY KEY,
-- The report's title
         title      TEXT        NOT NULL,
-- The first day of the range
         date_from  DATE        NOT NULL,
-- The last day of the range
         date_to    DATE        NOT NULL,
-- How the report is exported (the Figma design shows PDF or CSV; we change our mind in 3.5)
         format     TEXT        NOT NULL DEFAULT 'PDF' CHECK (format IN ('PDF', 'CSV')),
-- Which user created it
         created_by BIGINT      REFERENCES users (id) ON DELETE SET NULL,
-- When it was made
         created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
-- The start can't be after the end
         CHECK (date_from <= date_to)
);

-- Indexes for the most common lookups
-- We often sort leads by when they were added
CREATE INDEX idx_leads_created_at      ON leads (created_at);
-- We often filter deals by stage
CREATE INDEX idx_deals_stage           ON deals (stage);
-- We often find a customer's deals
CREATE INDEX idx_deals_customer_id     ON deals (customer_id);
-- We often sort activities by due time
CREATE INDEX idx_activities_due_at     ON activities (due_at);