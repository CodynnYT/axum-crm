
-- Five pretend leads
INSERT INTO leads (name, company, email, phone, source, stage, status) VALUES
('Sarah Wilson',  'Acme Labs',  'sarah@acmelabs.example',    '+1 555 0101', 'Website',      'New',         'Active'),
('Daniel Park',   'Brightpath', 'daniel@brightpath.example', '+1 555 0102', 'Referral',     'Contacted',   'Active'),
('Emma Collins',  'Northwind',  'emma@northwind.example',    '+1 555 0103', 'Social Media', 'Qualified',   'Active'),
('Lucas Meyer',   'Bluepeak',   'lucas@bluepeak.example',    '+1 555 0104', 'Cold Call',    'New',         'Active'),
('Olivia Grant',  'Fieldstone', 'olivia@fieldstone.example', '+1 555 0105', 'Email',        'Unqualified', 'Inactive');

-- Three pretend customers
INSERT INTO customers (name, company, email, phone, notes) VALUES
('Alex Morgan',  'Globex',         'alex@globex.example',   '+1 555 0201', 'Prefers email.'),
('Maya Chen',    'Initech',        'maya@initech.example',  '+1 555 0202', ''),
('John Carter',  'Umbrella Works', 'john@umbrella.example', '+1 555 0203', 'Decision maker.');

-- Nine deals spread over the last 6 months, so the chart has a shape
INSERT INTO deals (name, customer_id, amount, stage, expected_close, created_at) VALUES
     ('Globex annual plan',  1, 12000, 'Won',         CURRENT_DATE - 150, NOW() - INTERVAL '160 days'),
     ('Initech pilot',       2,  4500, 'Won',         CURRENT_DATE - 100, NOW() - INTERVAL '110 days'),
     ('Umbrella onboarding', 3,  8200, 'Won',         CURRENT_DATE - 60,  NOW() - INTERVAL '70 days'),
     ('Globex add-on',       1,  3200, 'Won',         CURRENT_DATE - 20,  NOW() - INTERVAL '30 days'),
     ('Initech expansion',   2,  9800, 'Negotiation', CURRENT_DATE + 15,  NOW() - INTERVAL '12 days'),
     ('Umbrella support',    3,  2600, 'Proposal',    CURRENT_DATE + 30,  NOW() - INTERVAL '8 days'),
     ('Portal rebuild',      NULL, 5400, 'Qualified', CURRENT_DATE + 45,  NOW() - INTERVAL '4 days'),
     ('Hardware bundle',     NULL, 1900, 'New',       CURRENT_DATE + 60,  NOW() - INTERVAL '1 day'),
     ('Old renewal',         1,  2100, 'Lost',        CURRENT_DATE - 40,  NOW() - INTERVAL '90 days');

-- Five activities: two done, three coming up
INSERT INTO activities (title, related_to, activity_type, due_at, status, notes) VALUES
     ('Call with Sarah Wilson', 'Sarah Wilson', 'Call',    NOW() - INTERVAL '15 minutes', 'Completed', 'Interested in a demo.'),
     ('Send proposal to Maya',  'Maya Chen',    'Email',   NOW() - INTERVAL '2 hours',    'Completed', ''),
     ('Demo meeting with John', 'John Carter',  'Meeting', NOW() + INTERVAL '3 hours',    'Pending',   ''),
     ('Follow up with Daniel',  'Daniel Park',  'Call',    NOW() + INTERVAL '1 day',      'Pending',   ''),
     ('Prepare contract',       'Initech',      'Task',    NOW() + INTERVAL '2 days',     'Pending',   '');