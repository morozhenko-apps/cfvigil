// Content-dependent tray height, independent of the previous window size.
// Native code clamps the result and owns the bottom-anchored resize.
export function expandedCardHeight({
  rowsHeight, warningHeight=0, searchHeight, actionsHeight,
  paddingTop=10, paddingBottom=10, borderTop=1, borderBottom=1,
  gap=7, drawerBorder=1, minHeight=142, maxHeight=510
}) {
  const parts=[rowsHeight,warningHeight,searchHeight,actionsHeight,
    paddingTop,paddingBottom,borderTop,borderBottom,gap,drawerBorder,minHeight,maxHeight];
  if (parts.some(value=>!Number.isFinite(value)||value<0) || maxHeight<minHeight) {
    throw Error("Invalid card content measurements");
  }
  // Three vertically stacked sections: drawer, search and five actions.
  const natural=rowsHeight+warningHeight+searchHeight+actionsHeight+
    paddingTop+paddingBottom+borderTop+borderBottom+drawerBorder+2*gap;
  return Math.ceil(Math.max(minHeight,Math.min(maxHeight,natural)));
}

export function measureExpandedCardHeight(doc=document) {
  const card=doc.getElementById("card");
  const list=doc.getElementById("resource-list");
  const warning=doc.getElementById("provider-warning");
  const search=doc.getElementById("search-row");
  const actions=card.querySelector(".actions");
  const styles=doc.defaultView.getComputedStyle(card);
  const size=value=>parseFloat(value)||0;
  return expandedCardHeight({
    rowsHeight:Array.from(list.children).reduce((sum,node)=>sum+node.getBoundingClientRect().height,0),
    warningHeight:warning.hidden?0:warning.getBoundingClientRect().height,
    searchHeight:search.getBoundingClientRect().height,
    actionsHeight:actions.getBoundingClientRect().height,
    paddingTop:size(styles.paddingTop),
    paddingBottom:size(styles.paddingBottom),
    borderTop:size(styles.borderTopWidth),
    borderBottom:size(styles.borderBottomWidth),
    gap:size(styles.rowGap),
    drawerBorder:size(doc.defaultView.getComputedStyle(doc.getElementById("drawer")).borderBottomWidth)
  });
}
