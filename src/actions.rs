use casbin::error::AdapterError;
use casbin::error::Error as CasbinError;
use casbin::Result;
use rbatis::executor::Executor;
use rbatis::py_sql;
use rbatis::Rbatis;
use rbs::to_value;

// use crate::casbin_rbatis_adapter::tables::CasbinRule;
use crate::adapter::TABLE_NAME;
use crate::tables::CasbinRule;

pub async fn new(rb: &rbatis::Rbatis) -> Result<()> {
    let driver_type = rb.driver_type().unwrap();

    let sql_statment = if "postgres" == driver_type {
        format!(
            "
                CREATE TABLE IF NOT EXISTS {} (
                    id bigserial PRIMARY KEY,
                    ptype character(12) NOT NULL,
                    v0 character(128) NOT NULL,
                    v1 character(128) NOT NULL,
                    v2 character(128) NOT NULL,
                    v3 character(128) NOT NULL,
                    v4 character(128) NOT NULL,
                    v5 character(128) NOT NULL
                )
            ",
            TABLE_NAME
        )
    } else {
        format!(
            "
                CREATE TABLE IF NOT EXISTS {} (
                    id INT NOT NULL AUTO_INCREMENT,
                    ptype VARCHAR(12) NOT NULL,
                    v0 VARCHAR(128) NOT NULL,
                    v1 VARCHAR(128) NOT NULL,
                    v2 VARCHAR(128) NOT NULL,
                    v3 VARCHAR(128) NOT NULL,
                    v4 VARCHAR(128) NOT NULL,
                    v5 VARCHAR(128) NOT NULL,
                    PRIMARY KEY(id),
                    CONSTRAINT unique_key_casbin_rbatis_adapter UNIQUE(ptype, v0, v1, v2, v3, v4, v5)
                ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
            ",
            TABLE_NAME
        )
    };

    rb.fetch_decode(&sql_statment, vec![])
        .await
        .map_err(|err| CasbinError::from(AdapterError(Box::new(err))))
}

pub(crate) async fn clear_policy(rb: &Rbatis) -> Result<()> {
    let name = TABLE_NAME.to_string();
    let sql_statment = format!("delete from {}", name);

    rb.fetch_decode(sql_statment.as_str(), vec![])
        .await
        .map_err(|err| CasbinError::from(AdapterError(Box::new(err))))?;
    Result::Ok(())
}

pub(crate) async fn save_policy(rb: &Rbatis, rules: Vec<CasbinRule>) -> Result<()> {
    let mut tx = rb.acquire_begin().await.map_err(|err| CasbinError::from(AdapterError(Box::new(err))))?;

    for rule in rules {
        CasbinRule::insert(&mut tx, &rule)
            .await
            .map_err(|err| CasbinError::from(AdapterError(Box::new(err))))?;
    }
    tx.commit().await.map_err(|err| CasbinError::from(AdapterError(Box::new(err))))?;
    Ok(())
}

#[py_sql(
    "`delete from casbin_rule`
        where:
            for k,rule in rules:
                ` ptype = #{ptype}`
                for key,item in rule:
                    ` and v${key} = #{item}` "
)]
async fn remove_policies_sql(rb: &mut dyn Executor, ptype: &str, rules: &Vec<Vec<String>>) -> rbatis::Result<()> {}

pub async fn remove_policy(rb: &Rbatis, pt: &str, rule: Vec<String>) -> Result<bool> {
    remove_policies(rb, pt, vec![rule]).await
}

pub async fn remove_policies(rb: &Rbatis, pt: &str, rules: Vec<Vec<String>>) -> Result<bool> {
    let mut normal_rules = vec![];

    for rule in rules {
        normal_rules.push(normalize_casbin_rule(rule, 0))
    }

    remove_policies_sql(&mut rb.clone(), pt, &normal_rules)
        .await
        .map_err(|err| CasbinError::from(AdapterError(Box::new(err))))
        .and_then(|_| Ok(true))
}

pub async fn remove_filtered_policy(rb: &Rbatis, pt: &str, field_index: usize, field_values: Vec<String>) -> Result<bool> {
    let field_values = normalize_casbin_rule(field_values, field_index);

    let (sql, parameters) = if field_index == 5 {
        let sql = "DELETE FROM casbin_rule WHERE ptype = ? AND (v5 is NULL OR v5 = COALESCE(?,v5))";
        let p = vec![to_value!(pt.to_string()), to_value!(field_values[0].to_string())];
        (sql, p)
    } else if field_index == 4 {
        let sql = "DELETE FROM casbin_rule WHERE
        ptype = ? AND
        (v4 is NULL OR v4 = COALESCE(?,v4)) AND
        (v5 is NULL OR v5 = COALESCE(?,v5))";

        let p = vec![
            to_value!(pt.to_string()),
            to_value!(field_values[0].to_string()),
            to_value!(field_values[1].to_string()),
        ];
        (sql, p)
    } else if field_index == 3 {
        let sql = "DELETE FROM casbin_rule WHERE
        ptype = ? AND
        (v3 is NULL OR v3 = COALESCE(?,v3)) AND
        (v4 is NULL OR v4 = COALESCE(?,v4)) AND
        (v5 is NULL OR v5 = COALESCE(?,v5))";
        let p = vec![
            to_value!(pt.to_string()),
            to_value!(field_values[0].to_string()),
            to_value!(field_values[1].to_string()),
            to_value!(field_values[2].to_string()),
        ];
        (sql, p)
    } else if field_index == 2 {
        let sql = "DELETE FROM casbin_rule WHERE
        ptype = ? AND
        (v2 is NULL OR v2 = COALESCE(?,v2)) AND
        (v3 is NULL OR v3 = COALESCE(?,v3)) AND
        (v4 is NULL OR v4 = COALESCE(?,v4)) AND
        (v5 is NULL OR v5 = COALESCE(?,v5))";
        let p = vec![
            to_value!(pt.to_string()),
            to_value!(field_values[0].to_string()),
            to_value!(field_values[1].to_string()),
            to_value!(field_values[2].to_string()),
            to_value!(field_values[3].to_string()),
        ];
        (sql, p)
    } else if field_index == 1 {
        let sql = "DELETE FROM casbin_rule WHERE
        ptype = ? AND
        (v1 is NULL OR v1 = COALESCE(?,v1)) AND
        (v2 is NULL OR v2 = COALESCE(?,v2)) AND
        (v3 is NULL OR v3 = COALESCE(?,v3)) AND
        (v4 is NULL OR v4 = COALESCE(?,v4)) AND
        (v5 is NULL OR v5 = COALESCE(?,v5))";
        let p = vec![
            to_value!(pt.to_string()),
            to_value!(field_values[0].to_string()),
            to_value!(field_values[1].to_string()),
            to_value!(field_values[2].to_string()),
            to_value!(field_values[3].to_string()),
            to_value!(field_values[4].to_string()),
        ];
        (sql, p)
    } else {
        let sql = "DELETE FROM casbin_rule WHERE
        ptype = ? AND
        (v0 is NULL OR v0 = COALESCE(?,v0)) AND
        (v1 is NULL OR v1 = COALESCE(?,v1)) AND
        (v2 is NULL OR v2 = COALESCE(?,v2)) AND
        (v3 is NULL OR v3 = COALESCE(?,v3)) AND
        (v4 is NULL OR v4 = COALESCE(?,v4)) AND
        (v5 is NULL OR v5 = COALESCE(?,v5))";
        let p = vec![
            to_value!(pt.to_string()),
            to_value!(field_values[0].to_string()),
            to_value!(field_values[1].to_string()),
            to_value!(field_values[2].to_string()),
            to_value!(field_values[3].to_string()),
            to_value!(field_values[4].to_string()),
            to_value!(field_values[5].to_string()),
        ];
        (sql, p)
    };
    rb.fetch_decode::<bool>(sql, parameters)
        .await
        .map_err(|err| CasbinError::from(AdapterError(Box::new(err))))
}

pub(crate) async fn load_policy(rb: &mut Rbatis) -> Result<Vec<CasbinRule>> {
    let vec_rules = CasbinRule::select_all(rb).await.map_err(|err| CasbinError::from(AdapterError(Box::new(err))))?;
    Result::Ok(vec_rules)
}

pub(crate) async fn add_policy(rb: &mut Rbatis, new_rule: CasbinRule) -> Result<bool> {
    CasbinRule::insert(rb, &new_rule)
        .await
        .map_err(|err| CasbinError::from(AdapterError(Box::new(err))))?;
    Result::Ok(true)
}

pub(crate) async fn add_policies(rb: &Rbatis, rules: Vec<CasbinRule>) -> Result<bool> {
    let mut tx = rb.acquire_begin().await.map_err(|err| CasbinError::from(AdapterError(Box::new(err))))?;

    for rule in rules {
        CasbinRule::insert(&mut tx, &rule)
            .await
            .map_err(|err| CasbinError::from(AdapterError(Box::new(err))))?;
    }
    tx.commit().await.map_err(|err| CasbinError::from(AdapterError(Box::new(err))))
}

fn normalize_casbin_rule(mut rule: Vec<String>, field_index: usize) -> Vec<String> {
    rule.resize(6 - field_index, String::from(""));
    rule
}

#[cfg(test)]
mod test {

    use super::normalize_casbin_rule;

    //cargo test casbin_rbatis_adapter::actions::test::test_normalize_casbin_rule --  --nocapture
    #[test]
    fn test_normalize_casbin_rule() {
        let rule = vec!["bob".to_string(), "data2".to_string(), "write".to_string()];
        let new_rule = normalize_casbin_rule(rule, 0);
        println!("{:?}", new_rule);
    }
}
