-- UUID

-- Restaurants

CREATE TABLE IF NOT EXISTS restaurants(
		restaurant_id UUID PRIMARY KEY,
		name VARCHAR(400) NOT NULL,
		url TEXT NOT NULL,
		city VARCHAR(200),
		coordinates VARCHAR(20),
		opening_hours VARCHAR(100),
		created_at TIMESTAMP DEFAULT NOW(),
		updated_at TIMESTAMP
);

-- Meals

CREATE TABLE IF NOT EXISTS meals(
		meal_id UUID PRIMARY KEY,
		meal_type TEXT NOT NULL,
		foodies	TEXT,
		date VARCHAR(500),
		scraped_at TIMESTAMP DEFAULT NOW(),
		restaurant_id UUID NOT NULL REFERENCES restaurants(restaurant_id)
);

-- Schools

CREATE TABLE IF NOT EXISTS schools(
		school_id UUID PRIMARY KEY,
		long_name VARCHAR(500) NOT NULL,
		name VARCHAR(200) NOT NULL,
		coordinates VARCHAR(20)
);

-- Search keywords (inverted index)

CREATE TABLE IF NOT EXISTS keywords(
		keyword_id UUID PRIMARY KEY,
		keyword VARCHAR(500) NOT NULL,
		restaurant_id UUID NOT NULL REFERENCES restaurants(restaurant_id),
		category VARCHAR(100) NOT NULL
);
