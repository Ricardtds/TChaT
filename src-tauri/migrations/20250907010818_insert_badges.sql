-- Add migration script here
INSERT INTO 
    badges(id, type, text)
VALUES
    (1, "og","OG"),
    (2, "bot", "Bot"),
    (3, "vip", "VIP"),
    (4, "founder", "Founder"),
    (5, "moderator","Moderator"),
    (6, "sub_gifter", "Sub Gifter"),
    (7, "subscriber", "Subscriber"),
    (8, "broadcaster","Broadcaster"),
    (9, "verified", "Verified channel");