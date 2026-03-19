-- admins

CREATE TABLE IF NOT EXISTS admins(
		admin_id UUID PRIMARY KEY,
		ssh_key TEXT NOT NULL,
		name TEXT NOT NULL UNIQUE
);

-- Scrape history

CREATE TABLE IF NOT EXISTS scrape_batch(
		batch_id UUID PRIMARY KEY,
		entity VARCHAR(400) NOT NULL,
		author UUID NOT NULL REFERENCES admins(admin_id),
		scraped_at TIMESTAMP DEFAULT NOW(),
		region VARCHAR(500) NOT NULL
);

-- Restaurants

CREATE TABLE IF NOT EXISTS restaurants(
		restaurant_id VARCHAR(400) PRIMARY KEY,
		name VARCHAR(400) NOT NULL,
		url TEXT NOT NULL,
		city VARCHAR(200),
		coordinates VARCHAR(100),
		opening_hours VARCHAR(400),
		created_at TIMESTAMP DEFAULT NOW(),
		updated_at TIMESTAMP,
		batch_id UUID NOT NULL REFERENCES scrape_batch(batch_id)
);

-- Meals

CREATE TABLE IF NOT EXISTS meals(
		meal_id UUID PRIMARY KEY,
		meal_type TEXT NOT NULL,
		foodies	TEXT,
		date VARCHAR(500),
		restaurant_id VARCHAR(400) NOT NULL REFERENCES restaurants(restaurant_id),
		batch_id UUID NOT NULL REFERENCES scrape_batch(batch_id)
);

-- Schools

CREATE TABLE IF NOT EXISTS schools(
		school_id UUID PRIMARY KEY,
		long_name VARCHAR(500) NOT NULL,
		name VARCHAR(200) NOT NULL,
		coordinates VARCHAR(20),
		batch_id UUID NOT NULL REFERENCES scrape_batch(batch_id)
);

-- Search keywords (inverted index)

CREATE TABLE IF NOT EXISTS keywords(
		keyword_id UUID PRIMARY KEY,
		keyword VARCHAR(500) NOT NULL,
		restaurant_id VARCHAR(400) NOT NULL REFERENCES restaurants(restaurant_id),
		category VARCHAR(100) NOT NULL
);

