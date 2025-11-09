CREATE TABLE `order`
(
    `id`      bigint       NOT NULL,
    `product` varchar(255) NOT NULL,
    `count`   int NULL DEFAULT NULL,
    `amount`  int NULL DEFAULT NULL,
    PRIMARY KEY (`id`) USING BTREE
);
