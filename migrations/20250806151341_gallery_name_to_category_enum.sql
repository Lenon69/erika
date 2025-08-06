-- Zdefiniowanie nowego typu ENUM z kategoriami
CREATE TYPE gallery_category AS ENUM (
    'Piersi',
    'Tyłek',
    'Stopy',
    'Nogi',
    'Twarz',
    'Bielizna',
    'Całe Ciało',
    'Artystyczne'
);

-- Zmiana typu kolumny 'name' na nowy ENUM
-- `USING name::gallery_category` próbuje rzutować istniejące wartości,
-- może wymagać ręcznej poprawy danych, jeśli były tam niestandardowe nazwy.
ALTER TABLE galleries
ALTER COLUMN name TYPE gallery_category
USING name::text::gallery_category;
