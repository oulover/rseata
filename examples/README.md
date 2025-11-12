设置数据库

* 创建两个数据库：order和user
    * 运行示例中的SQL脚本
        1. examples/user-service/user.sql
        2. examples/user-service/user.sql
    * 数据库连接环境变量：
        1. USER_DATABASE_URL=mysql://root:root@127.0.0.1:3306/user
        2. ORDER_DATABASE_URL=mysql://root:root@127.0.0.1:3306/order

* 运行
    ```shell
    cd user-service
    cargo run
   ```
  
    ```shell
    cd order-service
    cargo run
   ```

* 测试分布式事务

    通过order-service的API创建订单，order-service会调用user-service添加用户，这两个操作会在一个全局事务中。
    
    get 访问接口 http://127.0.0.1:4002/add_order_then_add_user