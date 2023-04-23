sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/Filter",
  "sap/ui/model/FilterOperator",
  "sap/m/MessageToast"
], function (BaseController, Filter, FilterOperator, MessageToast) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.ScoringTable", {

    onInit: async function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this._oTable = this.getView().byId("clubsTable");

      this._oScoringModel = await this.getJSONModel("/api/regattas/" + this.getRegattaId() + "/clubs", this._oTable);
      this.setViewModel(this._oScoringModel, "clubs");

      this.getRouter().getRoute("clubs").attachMatched(async (_) => await this._loadModel(), this);
    },

    onNavBack: function () {
      this.navBack("startpage");
    },

    onFilterSearch: function (oEvent) {
      const aSearchFilters = [];
      const sQuery = oEvent.getParameter("query").trim();
      if (sQuery) {
        aSearchFilters.push(
          new Filter({
            filters: [
              new Filter("shortName", FilterOperator.Contains, sQuery),
              new Filter("longName", FilterOperator.Contains, sQuery),
              new Filter("abbreviation", FilterOperator.Contains, sQuery),
              new Filter("city", FilterOperator.Contains, sQuery)
            ],
            and: false
          }))
      }
      const oBinding = this._oTable.getBinding("items");
      oBinding.filter(aSearchFilters);
    },

    onRefreshButtonPress: async function (oEvent) {
      await this._loadModel();
      MessageToast.show(this.i18n("msg.dataUpdated", undefined));
    },

    _loadModel: async function () {
      await this.updateJSONModel(this._oScoringModel, "/api/regattas/" + this.getRegattaId() + "/clubs", this._oTable)
    }

  });
});