SELECT emoji, COUNT(1), MAX(uid = ?2)
FROM Reactions
WHERE slug = ?1
GROUP BY emoji
