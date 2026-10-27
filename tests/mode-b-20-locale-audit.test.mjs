import test from "node:test";
import assert from "node:assert/strict";
import english from "../apps/desktop/ui/locales/en.mjs";
import {
  ACTIVE_LOCALES, PLANNED_LOCALES, catalogs, LANGUAGE_NAMES, validateCatalog,
  intlLocale, directionFor, t, plural, tokens, eventText, incidentText,
  diagnosticText, formatDate, relativeTime, localizeDocument
} from "../apps/desktop/ui/localization.mjs";

const SCOPES=["Workers Scripts Read","Workers CI Read","Pages Read",
  "Account Analytics Read","Zone Read","Zone Analytics Read"];
const STATES={
  en:["Successful","Deploying","Issues","Unverified"],
  ru:["Успешно","В процессе","Проблемы","Не проверено"],
  es:["Exitosos","En curso","Problemas","Sin verificar"],
  pt:["Sucesso","Em curso","Problemas","Não verificado"],
  de:["Erfolgreich","Läuft","Probleme","Nicht verifiziert"],
  fr:["Réussis","En cours","Problèmes","Non vérifié"],
  it:["Riusciti","In corso","Problemi","Non verificato"],
  pl:["Zakończone pomyślnie","W toku","Problemy","Niezweryfikowane"],
  hi:["सफल","प्रक्रिया जारी","समस्याएँ","सत्यापित नहीं"],
  "zh-Hant":["成功","進行中","問題","尚未驗證"],
  ja:["成功","進行中","問題","未確認"],
  nl:["Geslaagd","Bezig","Problemen","Niet geverifieerd"],
  sv:["Lyckade","Pågår","Problem","Ej verifierad"],
  tr:["Başarılı","Devam ediyor","Sorunlar","Doğrulanmadı"],
  ko:["성공","진행 중","문제","확인되지 않음"],
  bn:["সফল","চলমান","সমস্যা","যাচাই করা হয়নি"],
  th:["สำเร็จ","กำลังดำเนินการ","ปัญหา","ยังไม่ยืนยัน"],
  id:["Berhasil","Sedang berjalan","Masalah","Belum terverifikasi"],
  fil:["Matagumpay","Isinasagawa","Mga problema","Hindi pa beripikado"],
  ar:["ناجح","قيد التنفيذ","مشكلات","غير متحقق منه"]
};
const KEYS=["card.healthy","card.deploying","card.issues","card.statusUnknown"];

test("every supported locale has four distinct contextual Cloudflare status labels",()=>{
  assert.deepEqual(ACTIVE_LOCALES,PLANNED_LOCALES);
  assert.equal(ACTIVE_LOCALES.length,20);
  assert.deepEqual(Object.keys(STATES),ACTIVE_LOCALES);
  for(const locale of ACTIVE_LOCALES){
    assert.deepEqual(KEYS.map(key=>t(key,{},locale)),STATES[locale],locale);
    assert.equal(new Set(STATES[locale]).size,4,locale);
    assert.deepEqual(validateCatalog(catalogs[locale],english,locale),[],locale);
    assert.ok(LANGUAGE_NAMES[locale].length>0,locale);
  }
  assert.equal(t("card.healthy",{},"en"),"Successful");
  assert.equal(intlLocale("pt"),"pt-PT");
  assert.equal(intlLocale("zh-Hant"),"zh-Hant");
  assert.equal(directionFor("ar"),"rtl");
  assert.equal(ACTIVE_LOCALES.filter(lang=>directionFor(lang)==="rtl").length,1);
});

test("all twenty permission explanations preserve exact account scopes and Arabic isolation",()=>{
  for(const locale of ACTIVE_LOCALES){
    const help=t("settings.tokenHelp",{},locale);
    for(const scope of SCOPES){
      assert.equal(help.split(scope).length,2,locale+":"+scope);
      if(locale==="ar")
        assert.ok(help.includes("\u2068"+scope+"\u2069"),locale+":"+scope);
    }
    const placeholder=t("settings.accountPlaceholder",{},locale);
    assert.ok(/32|३२|৩২/.test(placeholder),locale);
  }
});

test("all dictionary messages render under production interpolation and CLDR rules",()=>{
  const params={name:"worker-a",scope:"Pages Read",version:"0.1.0",
    code:"10000",recent:150,baseline:20,errors:30,requests:200,
    rate:15,count:3,shown:2,total:3};
  for(const locale of ACTIVE_LOCALES){
    assert.deepEqual(Object.keys(catalogs[locale]).sort(),
      Object.keys(english).sort(),locale);
    for(const [key,value] of Object.entries(catalogs[locale])){
      if(typeof value==="string"){
        const args=Object.fromEntries(tokens(value).map(name=>
          [name,params[name]??"example"]));
        const rendered=t(key,args,locale);
        assert.ok(rendered.trim().length>0,locale+":"+key);
        assert.ok(!/\{[a-z][a-zA-Z0-9_]*\}/.test(rendered),locale+":"+key);
      }else{
        const categories=new Intl.PluralRules(intlLocale(locale))
          .resolvedOptions().pluralCategories;
        assert.deepEqual(Object.keys(value).sort(),categories.sort(),locale+":"+key);
        for(const count of [0,1,2,3,5,11,21,1000000]){
          const rendered=plural(key,count,params,locale);
          assert.ok(rendered.length>0,locale+":"+key+":"+count);
          assert.ok(!/\{[a-z][a-zA-Z0-9_]*\}/.test(rendered),locale+":"+key);
        }
        for(const bad of [-1,NaN,Infinity,-Infinity])
          assert.throws(()=>plural(key,bad,params,locale),/Invalid plural count/);
      }
    }
  }
});

test("provider errors and historical incident messages remain scoped in all locales",()=>{
  for(const locale of ACTIVE_LOCALES){
    for(const [name,status,code,scope] of [
      ["Pages",400,"8000024",null],
      ["Pages",403,"10000","Cloudflare Pages Read"],
      ["Workers Builds",403,"10000","Workers CI Read"],
      ["Workers Builds",429,"1015",null],
      ["Worker deployments",500,"10001",null]
    ]){
      const raw=name+": Cloudflare HTTP "+status+" (code "+code+")"+
        (scope?" (check "+scope+")":"");
      const result=diagnosticText(raw,locale);
      assert.ok(result.startsWith(name+": Cloudflare HTTP "+status),locale);
      assert.ok(result.includes(code),locale);
      assert.equal(result.includes("Cloudflare Pages Read")||
        result.includes("Workers CI Read"),Boolean(scope),locale+":"+status);
      if(scope && locale==="ar")
        assert.ok(result.includes("\u2068"+scope+"\u2069"));
    }
    const old="Latest Cloudflare build/deployment failed";
    assert.equal(eventText(old,locale),t("event.buildFailure",{},locale));
    assert.equal(eventText("Latest Cloudflare deployment succeeded",locale),
      t("event.buildSuccess",{},locale));
    assert.equal(incidentText("worker-a: "+old,locale),
      "worker-a: "+t("event.buildFailure",{},locale));
    assert.equal(diagnosticText("unexpected provider response",locale),
      "unexpected provider response");
  }
});

test("fixed UTC timestamps at DST boundary use the selected locale",()=>{
  const a="2026-03-29T00:59:00Z",b="2026-03-29T01:01:00Z";
  for(const locale of ACTIVE_LOCALES){
    const formatter=new Intl.DateTimeFormat(intlLocale(locale),
      {dateStyle:"medium",timeStyle:"short"});
    assert.equal(formatDate(a,locale),formatter.format(Date.parse(a)),locale);
    assert.equal(formatDate(b,locale),formatter.format(Date.parse(b)),locale);
    assert.notEqual(formatDate(a,locale),formatDate(b,locale),locale);
    assert.equal(formatDate("invalid",locale),t("history.unknownTime",{},locale));
    assert.equal(relativeTime(a,Date.parse(b),locale),
      plural("age.minute",2,{},locale));
  }
});

test("WebView translation attributes and RTL direction are applied for all twenty locales",()=>{
  const fields=[
    ["data-i18n","card.healthy","textContent"],
    ["data-i18n-title","card.hideTitle","title"],
    ["data-i18n-placeholder","card.searchPlaceholder","placeholder"],
    ["data-i18n-aria-label","card.searchLabel","aria-label"]
  ];
  for(const locale of ACTIVE_LOCALES){
    const nodes=fields.map(([attribute,key])=>({
      attribute,key,textContent:"",title:"",placeholder:"",attributes:{},
      getAttribute(name){return name===this.attribute?this.key:null;},
      setAttribute(name,value){this.attributes[name]=value;}
    }));
    const doc={documentElement:{lang:"en",dir:"ltr"},
      querySelectorAll(selector){
        return nodes.filter(n=>selector==="["+n.attribute+"]");
      }};
    localizeDocument(doc,locale);
    assert.equal(doc.documentElement.lang,intlLocale(locale),locale);
    assert.equal(doc.documentElement.dir,directionFor(locale),locale);
    fields.forEach(([,key,target],i)=>{
      const actual=target==="aria-label"
        ?nodes[i].attributes[target]:nodes[i][target];
      assert.equal(actual,t(key,{},locale),locale+":"+key);
    });
  }
});
