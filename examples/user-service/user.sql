CREATE TABLE `user`
(
    `id`   bigint       NOT NULL,
    `name` varchar(255) NOT NULL,
    `age`  int NULL DEFAULT NULL,
    `sex`  int NULL DEFAULT NULL,
    PRIMARY KEY (`id`) USING BTREE
);
