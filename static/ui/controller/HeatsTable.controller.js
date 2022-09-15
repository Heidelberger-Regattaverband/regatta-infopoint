sap.ui.define([
  "sap/ui/core/mvc/Controller",
  "sap/ui/model/json/JSONModel",
  "../model/StateLabelFormatter",
  "../model/RaceLabelFormatter",
  "../model/HeatLabelFormatter",
  "../model/DayFormatter",
  "../model/TimeFormatter",
  "sap/f/library"
], function (Controller, JSONModel, StateLabelFormatter, RaceLabelFormatter, HeatLabelFormatter, DayFormatter, TimeFormatter, fioriLibrary) {
  "use strict";
  return Controller.extend("de.regatta_hd.infopoint.controller.HeatsTable", {

    stateLabelFormatter: StateLabelFormatter,

    raceLabelFormatter: RaceLabelFormatter,

    heatLabelFormatter: HeatLabelFormatter,

    dayFormatter: DayFormatter,

    timeFormatter: TimeFormatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());
    },

    onSelectionChange: function (oEvent) {
      var oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        let oBindingCtx = oSelectedItem.getBindingContext("heat");
        let oItem = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());
        // alert(JSON.stringify(oItem));

        let oModel = new JSONModel();
        oModel.loadData("/api/heats/" + oItem.id + "/registrations");

        this.getOwnerComponent().setModel(oModel, "heatRegistration");

        var oFCL = this.getView().getParent().getParent();
        oFCL.setLayout(fioriLibrary.LayoutType.TwoColumnsMidExpanded);
      }
    }

  });
});