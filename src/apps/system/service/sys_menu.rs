use anyhow::{anyhow, Result};
use chrono::{Local, NaiveDateTime};
use db::{
    common::res::{ListData, PageParams},
    system::{
        entities::{
            prelude::{SysMenu, SysRoleApi},
            sys_menu, sys_role_api,
        },
        models::sys_menu::{AddReq, EditReq, MenuResp, Meta, SearchReq, SysMenuTree, UserMenu},
    },
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, JoinType,
    ModelTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};

use super::super::service;
use crate::utils;

/// get_list 获取列表
/// page_params 分页参数
/// db 数据库连接 使用db.0
pub async fn get_sort_list(
    db: &DatabaseConnection,
    page_params: PageParams,
    search_req: SearchReq,
) -> Result<ListData<sys_menu::Model>> {
    let page_num = page_params.page_num.unwrap_or(1);
    let page_per_size = page_params.page_size.unwrap_or(u32::MAX as usize);
    //  生成查询条件
    let mut s = SysMenu::find();

    if let Some(x) = search_req.menu_name {
        if !x.is_empty() {
            s = s.filter(sys_menu::Column::MenuName.contains(&x));
        }
    }
    if let Some(x) = search_req.method {
        if !x.is_empty() {
            s = s.filter(sys_menu::Column::Method.eq(x));
        }
    }

    if let Some(x) = search_req.status {
        if !x.is_empty() {
            s = s.filter(sys_menu::Column::Status.eq(x));
        }
    }
    if let Some(x) = search_req.menu_type {
        if !x.is_empty() {
            s = s.filter(sys_menu::Column::MenuType.eq(x));
        }
    }
    if let Some(x) = search_req.begin_time {
        s = s.filter(sys_menu::Column::CreatedAt.gte(x));
    }
    if let Some(x) = search_req.end_time {
        s = s.filter(sys_menu::Column::CreatedAt.lte(x));
    }
    // 获取全部数据条数
    let total = s.clone().count(db).await?;
    // 分页获取数据
    let paginator = s
        .order_by_asc(sys_menu::Column::OrderSort)
        .paginate(db, page_per_size);
    let total_pages = paginator.num_pages().await?;
    let list = paginator.fetch_page(page_num - 1).await?;

    let res = ListData {
        list,
        total,
        total_pages,
        page_num,
    };
    Ok(res)
}

pub async fn get_auth_list(
    db: &DatabaseConnection,
    page_params: PageParams,
    search_req: SearchReq,
) -> Result<ListData<sys_menu::Model>> {
    let page_num = page_params.page_num.unwrap_or(1);
    let page_per_size = page_params.page_size.unwrap_or(u32::MAX as usize);
    //  生成查询条件
    let mut s = SysMenu::find().filter(sys_menu::Column::MenuType.eq("F"));

    if let Some(x) = search_req.menu_name {
        if !x.is_empty() {
            s = s.filter(sys_menu::Column::MenuName.contains(&x));
        }
    }
    if let Some(x) = search_req.method {
        if !x.is_empty() {
            s = s.filter(sys_menu::Column::Method.eq(x));
        }
    }

    if let Some(x) = search_req.status {
        if !x.is_empty() {
            s = s.filter(sys_menu::Column::Status.eq(x));
        }
    }
    if let Some(x) = search_req.begin_time {
        s = s.filter(sys_menu::Column::CreatedAt.gte(x));
    }
    if let Some(x) = search_req.end_time {
        s = s.filter(sys_menu::Column::CreatedAt.lte(x));
    }

    // 获取全部数据条数
    let total = s.clone().count(db).await?;
    // 分页获取数据
    let paginator = s
        .order_by_asc(sys_menu::Column::OrderSort)
        .paginate(db, page_per_size);
    let total_pages = paginator.num_pages().await?;
    let list = paginator.fetch_page(page_num - 1).await?;

    let res = ListData {
        list,
        total,
        total_pages,
        page_num,
    };
    Ok(res)
}

pub async fn check_router_is_exist_update(db: &DatabaseConnection, req: EditReq) -> Result<bool> {
    let s1 = SysMenu::find()
        .filter(sys_menu::Column::Path.eq(req.clone().path))
        .filter(sys_menu::Column::Pid.eq(req.clone().pid))
        .filter(sys_menu::Column::MenuType.ne("F"))
        .filter(sys_menu::Column::Id.ne(req.id.clone()));
    let count1 = s1.count(db).await?;
    let s2 = SysMenu::find()
        .filter(sys_menu::Column::MenuName.eq(req.menu_name.clone()))
        .filter(sys_menu::Column::MenuType.ne("F"))
        .filter(sys_menu::Column::Id.ne(req.id.clone()));
    let count2 = s2.count(db).await?;
    Ok(count1 > 0 || count2 > 0)
}

pub async fn check_router_is_exist_add<C>(db: &C, req: AddReq) -> Result<bool>
where
    C: TransactionTrait + ConnectionTrait,
{
    let s1 = SysMenu::find()
        .filter(sys_menu::Column::Path.eq(req.clone().path))
        .filter(sys_menu::Column::Pid.eq(req.clone().pid))
        .filter(sys_menu::Column::MenuType.ne("F"));
    let count1 = s1.count(db).await?;
    let s2 = SysMenu::find().filter(sys_menu::Column::MenuName.eq(req.clone().menu_name));
    let count2 = s2.count(db).await?;
    Ok(count1 > 0 || count2 > 0)
}

/// add 添加
pub async fn add<C>(db: &C, req: AddReq) -> Result<String>
where
    C: TransactionTrait + ConnectionTrait,
{
    //  检查数据是否存在
    if check_router_is_exist_add(db, req.clone()).await? {
        return Err(anyhow!("路径或者名称重复"));
    }
    let reqq = req.clone();
    let uid = scru128::scru128().to_string();
    let now: NaiveDateTime = Local::now().naive_local();
    let active_model = sys_menu::ActiveModel {
        id: Set(uid.clone()),
        pid: Set(req.pid),
        menu_name: Set(req.menu_name),
        icon: Set(req.icon.unwrap_or_else(|| "".to_string())),
        remark: Set(req.remark),
        menu_type: Set(req.menu_type),
        query: Set(req.query),
        api: Set(req.api),
        method: Set(req.method.unwrap_or_else(|| "".to_string())),
        order_sort: Set(req.order_sort),
        status: Set(req.status),
        visible: Set(req.visible),
        path: Set(req.path.unwrap_or_else(|| "".to_string())),
        component: Set(req.component.unwrap_or_else(|| "".to_string())),
        is_data_scope: Set(req.is_data_scope),
        is_frame: Set(req.is_frame),
        is_cache: Set(req.is_cache),
        created_at: Set(Some(now)),
        ..Default::default()
    };
    let txn = db.begin().await?;
    //  let re =   user.insert(db).await?; 这个多查询一次结果
    let _ = SysMenu::insert(active_model).exec(&txn).await?;
    txn.commit().await?;
    let res = format!("{} 添加成功", uid);
    // 添加api到全局
    utils::ApiUtils::add_api(reqq.api.as_str(), reqq.menu_name.as_str()).await;
    Ok(res)
}

/// delete 完全删除
pub async fn delete(db: &DatabaseConnection, id: String) -> Result<String> {
    let s = SysMenu::find()
        .filter(sys_menu::Column::Id.eq(id))
        .one(db)
        .await?;
    let txn = db.begin().await?;
    if let Some(m) = s {
        let api = m.clone().api;
        utils::ApiUtils::remove_api(api.as_str()).await;
        m.delete(db).await?;
    }
    txn.commit().await?;
    Ok("菜单删除成功".to_string())
}

// edit 修改
pub async fn edit(db: &DatabaseConnection, req: EditReq) -> Result<String> {
    //  检查数据是否存在
    if check_router_is_exist_update(db, req.clone()).await? {
        return Err(anyhow!("路径或者名称重复"));
    }
    let uid = req.id;
    let s_s = match SysMenu::find_by_id(uid.clone()).one(db).await? {
        Some(s) => s,
        None => return Err(anyhow!("菜单不存在")),
    };
    let s_y = s_s.clone();
    let s_r: sys_menu::ActiveModel = s_s.into();
    let now: NaiveDateTime = Local::now().naive_local();
    let act = sys_menu::ActiveModel {
        id: Set(uid.clone()),
        pid: Set(req.pid),
        menu_name: Set(req.menu_name),
        icon: Set(req.icon.unwrap_or_else(|| "".to_string())),
        remark: Set(req.remark),
        api: Set(req.api),
        method: Set(req.method.unwrap_or_else(|| "".to_string())),
        query: Set(req.query),
        menu_type: Set(req.menu_type),
        order_sort: Set(req.order_sort),
        status: Set(req.status),
        visible: Set(req.visible),
        path: Set(req.path),
        component: Set(req.component),
        is_data_scope: Set(req.is_data_scope),
        is_frame: Set(req.is_frame),
        is_cache: Set(req.is_cache),
        updated_at: Set(Some(now)),
        ..s_r
    };
    // 更新
    let up_model = act.update(db).await?; // 这个两种方式一样 都要多查询一次
    match &up_model.api == &s_y.api {
        true => {
            // 不更新api
        }
        false => {
            utils::ApiUtils::remove_api(&s_y.api).await;
            utils::ApiUtils::add_api(&up_model.api, &up_model.menu_name).await;
            service::sys_role_api::update_api(
                db,
                (&s_y.api, &s_y.method),
                (&up_model.api, &up_model.method),
            )
            .await?;
        }
    }

    let res = format!("{} 修改成功", uid);
    Ok(res)
}

/// get_user_by_id 获取用户Id获取用户
/// db 数据库连接 使用db.0
pub async fn get_by_id(db: &DatabaseConnection, search_req: SearchReq) -> Result<MenuResp> {
    let mut s = SysMenu::find();
    s = s.filter(sys_menu::Column::DeletedAt.is_null());
    //
    if let Some(x) = search_req.id {
        s = s.filter(sys_menu::Column::Id.eq(x));
    } else {
        return Err(anyhow!("请求参数错误"));
    }

    let res = match s.into_model::<MenuResp>().one(db).await? {
        Some(m) => m,
        None => return Err(anyhow!("数据不存在")),
    };

    Ok(res)
}

// 获取全部菜单 或者 除开按键api级别的外的路由
pub async fn get_menus<C>(db: &C, is_router: bool) -> Result<Vec<MenuResp>>
where
    C: TransactionTrait + ConnectionTrait,
{
    let mut s = SysMenu::find()
        .filter(sys_menu::Column::DeletedAt.is_null())
        .filter(sys_menu::Column::Status.eq("1"));
    if is_router {
        s = s.filter(sys_menu::Column::MenuType.ne("F"));
    };

    let res = s
        .order_by(sys_menu::Column::OrderSort, Order::Asc)
        .into_model::<MenuResp>()
        .all(db)
        .await?;
    Ok(res)
}

/// get_all 获取全部
/// db 数据库连接 使用db.0
pub async fn get_all_router_tree(db: &DatabaseConnection) -> Result<Vec<SysMenuTree>> {
    let menus = get_menus(db, true).await?;
    let menu_data = self::get_menu_data(menus);
    let menu_tree = self::get_menu_tree(menu_data, "0".to_string());

    Ok(menu_tree)
}

/// get_all 获取全部
/// db 数据库连接 使用db.0
pub async fn get_all_menu_tree(db: &DatabaseConnection) -> Result<Vec<SysMenuTree>> {
    let menus = get_menus(db, false).await?;
    let menu_data = self::get_menu_data(menus);
    let menu_tree = self::get_menu_tree(menu_data, "0".to_string());

    Ok(menu_tree)
}

//  获取角色对应的api 和 api id
// 返回结果(Vec<String>, Vec<String>) 为（apis,api_ids）
pub async fn get_role_permissions(
    db: &DatabaseConnection,
    role_ids: Vec<String>,
) -> Result<(Vec<String>, Vec<String>)> {
    let s = SysMenu::find()
        .join_rev(
            JoinType::InnerJoin,
            SysRoleApi::belongs_to(SysMenu)
                .from(sys_role_api::Column::Api)
                .to(sys_menu::Column::Api)
                .into(),
        )
        .filter(sys_role_api::Column::RoleId.is_in(role_ids))
        .all(db)
        .await?;

    let mut apis: Vec<String> = Vec::new();
    let mut api_ids: Vec<String> = Vec::new();
    for x in s {
        apis.push(x.api.clone());
        api_ids.push(x.id.clone());
    }
    Ok((apis, api_ids))
}

/// get_all 获取全部
/// db 数据库连接 使用db.0
pub async fn get_admin_menu_by_role_ids(
    db: &DatabaseConnection,
    role_ids: Vec<String>,
) -> Result<Vec<SysMenuTree>> {
    let (menu_apis, _) = self::get_role_permissions(db, role_ids).await?;
    //  todo 可能以后加条件判断
    let router_all = get_menus(db, true).await?;
    //  生成menus
    let mut menus: Vec<MenuResp> = Vec::new();
    for ele in router_all {
        if menu_apis.contains(&ele.api) {
            menus.push(ele);
        }
    }
    let menu_data = self::get_menu_data(menus);
    let menu_tree = self::get_menu_tree(menu_data, "0".to_string());
    Ok(menu_tree)
}

pub fn get_menu_tree(user_menus: Vec<SysMenuTree>, pid: String) -> Vec<SysMenuTree> {
    let mut menu_tree: Vec<SysMenuTree> = Vec::new();
    for mut user_menu in user_menus.clone() {
        if user_menu.user_menu.pid == pid {
            user_menu.children = Some(get_menu_tree(
                user_menus.clone(),
                user_menu.user_menu.id.clone(),
            ));
            menu_tree.push(user_menu.clone());
        }
    }
    menu_tree
}

//  整理菜单数据 // todo
pub fn get_menu_data(menus: Vec<MenuResp>) -> Vec<SysMenuTree> {
    let mut menu_res: Vec<SysMenuTree> = Vec::new();
    for mut menu in menus {
        menu.pid = menu.pid.trim().to_string();
        let meta = Meta {
            icon: menu.icon.clone(),
            title: menu.menu_name.clone(),
            hidden: menu.visible.clone() != "1",
            link: if menu.path.clone().starts_with("http") {
                Some(menu.path.clone())
            } else {
                None
            },
            no_cache: menu.is_cache.clone() != "1",
        };
        let user_menu = UserMenu {
            meta,
            id: menu.id.clone(),
            pid: menu.pid.clone(),
            path: if !menu.path.clone().starts_with('/') && menu.pid.clone() == "0" {
                format!("/{}", menu.path.clone())
            } else {
                menu.path.clone()
            },
            name: menu.path.clone(),
            menu_name: menu.menu_name.clone(),
            menu_type: menu.menu_type.clone(),
            always_show: if menu.is_cache.clone() == "1" && menu.pid.clone() == "0" {
                Some(true)
            } else {
                None
            },
            component: menu.component.clone(),
            hidden: menu.visible.clone() == "0",
        };
        let menu_tree = SysMenuTree {
            user_menu,
            ..Default::default()
        };
        menu_res.push(menu_tree);
    }
    menu_res
}
