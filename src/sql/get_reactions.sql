SELECT emoji, COUNT(1), CASE uid WHEN ?2 THEN true ELSE false END
FROM Reactions
WHERE slug = ?1
GROUP BY emoji
