-- Open tickets per project, newest first
/* Runs nightly; $1 is the cut-off date */
CREATE TABLE IF NOT EXISTS tracker.ticket (
    id BIGINT PRIMARY KEY,
    project_key VARCHAR(16) NOT NULL,
    title TEXT NOT NULL,
    labels TEXT[],
    priority priority_level DEFAULT 'medium',
    estimate NUMERIC(6, 2),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);

CREATE FUNCTION tracker.age_in_days(since TIMESTAMP) RETURNS INT AS $$
    SELECT extract(DAY FROM now() - since)::INT;
$$ LANGUAGE sql;

WITH recent AS (
    SELECT t.project_key, t.created_at, t.labels[1] AS label
    FROM tracker.ticket AS t
    WHERE t.created_at > $1 AND t.title LIKE '%login%'
)
SELECT r.project_key,
       count(*) AS opened,
       tracker.age_in_days(min(r.created_at)) AS oldest,
       sum(CASE WHEN r.created_at > ? THEN 1 ELSE 0 END) * 1.5 AS weight
FROM recent r
GROUP BY r.project_key
ORDER BY opened DESC
LIMIT 10;

UPDATE tracker.ticket SET estimate = estimate + 0.5 WHERE id = @ticket_id;
