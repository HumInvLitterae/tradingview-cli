use super::*;

#[tokio::test]
#[ignore = "run through scripts/check-account-js-contract.py with pinned Node.js"]
async fn javascript_account_indicator_input_ids() {
    let source =
        "indicator(\"Signals\")\nlength=input.int(20)\nalertcondition(close > open, \"Long\")";
    let mut runtime = FakeRuntime::new([saved_script_fixture(), json!({"source": source})]);
    let _ = alert_create_indicator(&mut runtime, source_check_request(source, false)).await;
    let expression = serde_json::to_string(&runtime.evaluated[2].0).unwrap();
    let script = format!(
        r#"
        const assert = require('node:assert/strict');
        const expression = {expression};
        const system = [
            {{id:'text',value:'synthetic-source'}}, {{id:'pineId',value:'synthetic-id'}},
            {{id:'pineVersion',value:'4.0'}}, {{id:'pineFeatures',value:'synthetic-features'}},
            {{id:'__fast_calc',value:true}}, {{id:'__profile',value:true}}
        ];
        const metadata = [...system.map(v => ({{id:v.id}})), {{id:'in_0'}}];
        const values = [...system.slice(0,4), {{id:'in_0',value:20}}, ...system.slice(4)];
        const cases = [
            ['system-prefix', metadata, values, {{in_0:20}}],
            ['reordered', [{{id:'in_0'}},{{id:'in_2'}}],
                [{{id:'in_2',value:false}},{{id:'in_0',value:0}}], {{in_0:0,in_2:false}}],
            ['string-array', [{{id:'in_0'}},{{id:'in_1'}}],
                [{{id:'in_0',value:''}},{{id:'in_1',value:[1,'x',false]}}], {{in_0:'',in_1:[1,'x',false]}}],
            ['value-alias', [{{id:'in_0'}}], [{{id:'in_0',val:20}}], {{in_0:20}}],
            ['no-user-inputs', system.map(v => ({{id:v.id}})), system, {{}}],
            ['missing-user-input', metadata, system, null],
            ['duplicate-value-id', metadata, [...values,{{id:'in_0',value:30}}], null],
            ['duplicate-declaration', [...metadata,{{id:'in_0'}}], values, null],
            ['undeclared-input', metadata, [...values,{{id:'in_1',value:30}}], null],
            ['default-only', [{{id:'in_0'}}], [{{id:'in_0',defval:20}}], null],
            ['missing-metadata', null, values, null],
            ['missing-values', metadata, null, null],
            ['malformed-declaration', [null], values, null],
            ['malformed-value', metadata, [...values,null], null],
            ['unknown-input-shape', [{{id:'unexpected'}}], [{{id:'unexpected',value:20}}], null],
            ['metadata-throws', 'throws', values, null],
            ['values-throws', metadata, 'throws', null]
        ];
        (async () => {{
            const results = [];
            for (const [name, declared, supplied, expected] of cases) {{
                let writes = 0, reads = 0, payload;
                const study = {{
                    _study: {{metaInfo: () => {{
                        if (declared === 'throws') throw new Error('synthetic-private-detail');
                        return {{inputs:declared}};
                    }}}},
                    getInputValues: () => {{
                        if (supplied === 'throws') throw new Error('synthetic-private-detail');
                        return supplied;
                    }}
                }};
                global.window = {{TradingViewApi:{{_activeChartWidgetWV:{{value:()=>({{
                    symbolExt:()=>({{symbol:'EXCHANGE:EXAMPLE'}}),
                    getAllStudies:()=>[{{id:'test-entity',name:'Signals'}}],
                    getStudyById:()=>study
                }})}}}}}};
                global.fetch = async (_, options) => {{
                    if (options.method === 'POST') {{
                        writes++;
                        payload = JSON.parse(options.body).payload;
                        return {{ok:true,text:async()=>'{{"s":"ok"}}'}};
                    }}
                    reads++;
                    return {{ok:true,json:async()=>({{r:payload ? [{{
                        id:'new',symbol:payload.symbol,message:payload.message,
                        condition:payload.conditions[0]
                    }}] : []}})}};
                }};
                const result = await eval(expression);
                if (expected !== null) {{
                    assert.equal(writes,1,name);
                    const sent = payload.conditions[0].series[0].inputs;
                    const user = Object.fromEntries(Object.entries(sent).filter(([id])=>/^in_\d+$/.test(id)));
                    assert.deepEqual(user,expected,name);
                    assert.deepEqual(Object.keys(sent).sort(),
                        [...Object.keys(expected),'pineFeatures','__fast_calc','__profile'].sort(),name);
                    assert.equal(sent.__fast_calc,false);
                    assert.equal(sent.__profile,false);
                    assert.equal(JSON.stringify(sent).includes('synthetic-'),false,name);
                    assert.equal(result.created,true,name);
                    assert.equal(result.input_metadata.input_count,Object.keys(expected).length,name);
                }} else {{
                    assert.equal(writes,0,name);
                    assert.equal(reads,0,name);
                    assert.equal(result.phase,'study_input_metadata_unavailable',name);
                    assert.equal(result.created,false,name);
                    assert.equal(JSON.stringify(result).includes('synthetic-private-detail'),false,name);
                }}
                results.push({{result,expected}});
            }}
            console.log(JSON.stringify(results));
        }})().catch(error=>{{console.error(error);process.exitCode=1;}});
        "#
    );
    let output = std::process::Command::new("node")
        .args(["-e", &script])
        .output()
        .expect("Node.js is required for the account JavaScript contract");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    for row in rows {
        let result = normalize_indicator_alert_create_payload(row["result"].clone());
        if let Some(expected) = row["expected"].as_object() {
            assert_eq!(
                result.unwrap()["input_metadata"]["input_count"],
                expected.len()
            );
        } else {
            let error = result.unwrap_err();
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert_eq!(error.details.unwrap()["created"], false);
        }
    }
}
