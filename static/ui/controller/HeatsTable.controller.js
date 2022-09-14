sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "../model/StateLabelFormatter",
  "../model/RaceLabelFormatter",
  "../model/HeatLabelFormatter"
], function (Controller, StateLabelFormatter, RaceLabelFormatter, HeatLabelFormatter) {
  "use strict";

  return Controller.extend("de.regatta_hd.infopoint.controller.HeatsTable", {
    stateLabelFormatter: StateLabelFormatter,

    raceLabelFormatter: RaceLabelFormatter,

    heatLabelFormatter: HeatLabelFormatter,

    onSelectionChange: function (oEvent) {
      var oSelectedItem = oEvent.getParameter("listItem");
      var oBindingCtx = oSelectedItem.getBindingContext("heat");
      var oItem = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());
      alert(JSON.stringify(oItem));
    }

  });
});